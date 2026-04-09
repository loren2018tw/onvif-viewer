use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::State as AxumState,
    http::Response,
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::{
    io::AsyncReadExt,
    net::TcpListener,
    process::{Child, Command},
    sync::{broadcast, oneshot},
};
use std::process::Stdio;

const BOUNDARY: &str = "frame";
const MAX_BUF_SIZE: usize = 4 * 1024 * 1024;

struct ActiveStream {
    ffmpeg: Child,
    shutdown_tx: oneshot::Sender<()>,
    reader_handle: tokio::task::JoinHandle<()>,
}

pub struct StreamManager {
    active: Option<ActiveStream>,
}

#[derive(Clone)]
struct MjpegState {
    frame_tx: Arc<broadcast::Sender<Vec<u8>>>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub async fn start(&mut self, rtsp_uri: &str) -> Result<String> {
        self.stop().await;

        // Verify ffmpeg is available
        Command::new("ffmpeg")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .context("找不到 ffmpeg，請先安裝 ffmpeg")?;

        // Spawn FFmpeg: RTSP → MJPEG pipe
        let mut ffmpeg = Command::new("ffmpeg")
            .args([
                "-rtsp_transport", "tcp",
                "-i", rtsp_uri,
                "-f", "image2pipe",
                "-c:v", "mjpeg",
                "-q:v", "5",
                "-r", "15",
                "-an",
                "pipe:1",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("無法啟動 ffmpeg")?;

        let stdout = ffmpeg.stdout.take().context("無法取得 ffmpeg stdout")?;

        // Broadcast channel for JPEG frames
        let (frame_tx, _) = broadcast::channel::<Vec<u8>>(30);
        let frame_tx = Arc::new(frame_tx);

        // Spawn reader task: parse JPEG frames from ffmpeg stdout
        let tx_clone = frame_tx.clone();
        let reader_handle = tokio::spawn(async move {
            read_jpeg_frames(stdout, tx_clone).await;
        });

        // Start local MJPEG HTTP server on random port
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let app = Router::new()
            .route("/stream", get(mjpeg_handler))
            .with_state(MjpegState { frame_tx: frame_tx.clone() });

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async { let _ = shutdown_rx.await; })
                .await
                .ok();
        });

        self.active = Some(ActiveStream {
            ffmpeg,
            shutdown_tx,
            reader_handle,
        });

        Ok(format!("http://127.0.0.1:{}/stream", port))
    }

    pub async fn stop(&mut self) {
        if let Some(mut active) = self.active.take() {
            let _ = active.shutdown_tx.send(());
            active.reader_handle.abort();
            let _ = active.ffmpeg.kill().await;
        }
    }
}

async fn read_jpeg_frames(
    mut stdout: tokio::process::ChildStdout,
    tx: Arc<broadcast::Sender<Vec<u8>>>,
) {
    let mut buf = Vec::with_capacity(512 * 1024);
    let mut temp = [0u8; 65536];

    loop {
        match stdout.read(&mut temp).await {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                buf.extend_from_slice(&temp[..n]);

                // Prevent unbounded memory growth
                if buf.len() > MAX_BUF_SIZE {
                    buf.clear();
                    continue;
                }

                // Extract and broadcast complete JPEG frames
                while let Some(frame) = extract_jpeg(&mut buf) {
                    let _ = tx.send(frame);
                }
            }
        }
    }
}

/// Extract a complete JPEG image from buffer (SOI 0xFFD8 → EOI 0xFFD9)
fn extract_jpeg(buf: &mut Vec<u8>) -> Option<Vec<u8>> {
    let soi = buf.windows(2).position(|w| w == [0xFF, 0xD8])?;
    let search = &buf[soi + 2..];
    let eoi_offset = search.windows(2).position(|w| w == [0xFF, 0xD9])?;
    let eoi = soi + 2 + eoi_offset + 2;

    let frame = buf[soi..eoi].to_vec();
    buf.drain(..eoi);
    Some(frame)
}

async fn mjpeg_handler(
    AxumState(state): AxumState<MjpegState>,
) -> Response<Body> {
    let mut rx = state.frame_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(frame) => {
                    let header = format!(
                        "--{}\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
                        BOUNDARY,
                        frame.len()
                    );
                    yield Ok::<_, std::convert::Infallible>(axum::body::Bytes::from(header.into_bytes()));
                    yield Ok(axum::body::Bytes::from(frame));
                    yield Ok(axum::body::Bytes::from("\r\n"));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Response::builder()
        .header(
            "Content-Type",
            format!("multipart/x-mixed-replace; boundary={}", BOUNDARY),
        )
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from_stream(stream))
        .unwrap()
}
