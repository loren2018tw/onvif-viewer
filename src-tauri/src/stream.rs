use crate::models::PreviewSession;
use anyhow::{Context, Result};
use axum::{
    body::Body,
    extract::{Path, State as AxumState},
    http::{Response, StatusCode},
    routing::get,
    Router,
};
use std::path::{Component, Path as StdPath, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    process::{Child, Command},
    sync::oneshot,
    time::{sleep, Duration, Instant},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

struct ActiveStream {
    output_dir: PathBuf,
    ffmpeg: Child,
    shutdown_tx: oneshot::Sender<()>,
}

fn hide_window(cmd: &mut Command) {
    #[cfg(windows)]
    {
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    {
        let _ = cmd;
    }
}

pub struct StreamManager {
    active: Option<ActiveStream>,
}

#[derive(Clone)]
struct PreviewState {
    output_dir: Arc<PathBuf>,
    session_id: Arc<String>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self { active: None }
    }

    pub async fn start(&mut self, rtsp_uri: &str) -> Result<PreviewSession> {
        self.stop().await;

        // Verify ffmpeg is available
        let mut version_cmd = Command::new("ffmpeg");
        version_cmd
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        hide_window(&mut version_cmd);
        version_cmd
            .status()
            .await
            .context("找不到 ffmpeg，請先安裝 ffmpeg")?;

        let session_id = uuid::Uuid::new_v4().to_string();
        let output_dir = std::env::temp_dir()
            .join("onvif-viewer-preview")
            .join(&session_id);
        std::fs::create_dir_all(&output_dir).context("無法建立預覽輸出目錄")?;
        let playlist_path = output_dir.join("index.m3u8");
        let segment_pattern = output_dir.join("segment_%03d.ts");
        let has_audio = probe_has_audio(rtsp_uri).await;

        // Spawn FFmpeg: RTSP → HLS (video + optional audio)
        let mut ffmpeg = Command::new("ffmpeg");
        ffmpeg
            .args([
                "-hide_banner",
                "-loglevel",
                "error",
                "-rtsp_transport",
                "tcp",
                "-i",
                rtsp_uri,
                "-map",
                "0:v:0",
                "-map",
                "0:a:0?",
                "-c:v",
                "libx264",
                "-preset",
                "ultrafast",
                "-tune",
                "zerolatency",
                "-pix_fmt",
                "yuv420p",
                "-g",
                "30",
                "-sc_threshold",
                "0",
                "-c:a",
                "aac",
                "-ar",
                "48000",
                "-b:a",
                "128k",
                "-f",
                "hls",
                "-hls_time",
                "1",
                "-hls_list_size",
                "3",
                "-hls_flags",
                "delete_segments+append_list+independent_segments+omit_endlist",
                "-hls_segment_filename",
            ])
            .arg(segment_pattern.as_os_str())
            .arg(playlist_path.as_os_str())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        hide_window(&mut ffmpeg);
        let mut ffmpeg = ffmpeg.spawn().context("無法啟動 ffmpeg")?;

        wait_for_playlist(&mut ffmpeg, &playlist_path).await?;

        // Start local HLS HTTP server on random port
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let preview_url = format!(
            "http://127.0.0.1:{}/preview/{}/index.m3u8",
            port, session_id
        );

        let state = PreviewState {
            output_dir: Arc::new(output_dir.clone()),
            session_id: Arc::new(session_id.clone()),
        };

        let app = Router::new()
            .route(
                "/preview/:session_id/index.m3u8",
                get(hls_file_handler_index),
            )
            .route("/preview/:session_id/*file", get(hls_file_handler))
            .with_state(state);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        self.active = Some(ActiveStream {
            output_dir,
            ffmpeg,
            shutdown_tx,
        });

        Ok(PreviewSession {
            session_id,
            preview_url,
            stream_uri: rtsp_uri.to_string(),
            has_audio,
        })
    }

    pub async fn stop(&mut self) {
        if let Some(mut active) = self.active.take() {
            let _ = active.shutdown_tx.send(());
            let _ = active.ffmpeg.kill().await;
            let _ = std::fs::remove_dir_all(&active.output_dir);
        }
    }
}

async fn probe_has_audio(rtsp_uri: &str) -> bool {
    let mut ffprobe = Command::new("ffprobe");
    ffprobe.args([
        "-v",
        "error",
        "-rtsp_transport",
        "tcp",
        "-select_streams",
        "a",
        "-show_entries",
        "stream=index",
        "-of",
        "csv=p=0",
        rtsp_uri,
    ]);
    hide_window(&mut ffprobe);

    match ffprobe.output().await {
        Ok(output) if output.status.success() => {
            !String::from_utf8_lossy(&output.stdout).trim().is_empty()
        }
        _ => false,
    }
}

async fn wait_for_playlist(ffmpeg: &mut Child, playlist_path: &StdPath) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if playlist_path.exists() {
            let metadata = std::fs::metadata(playlist_path).context("無法讀取預覽播放清單")?;
            if metadata.len() > 0 {
                return Ok(());
            }
        }

        if let Some(status) = ffmpeg.try_wait().context("無法確認 ffmpeg 狀態")? {
            anyhow::bail!("ffmpeg 預覽程序提前結束（狀態碼: {}）", status);
        }

        if Instant::now() >= deadline {
            anyhow::bail!("預覽來源建立逾時，尚未產生播放清單");
        }

        sleep(Duration::from_millis(200)).await;
    }
}

async fn hls_file_handler_index(
    Path(session_id): Path<String>,
    AxumState(state): AxumState<PreviewState>,
) -> Result<Response<Body>, StatusCode> {
    if session_id != *state.session_id {
        return Err(StatusCode::NOT_FOUND);
    }
    serve_file(&state.output_dir, "index.m3u8").await
}

async fn hls_file_handler(
    Path((session_id, file)): Path<(String, String)>,
    AxumState(state): AxumState<PreviewState>,
) -> Result<Response<Body>, StatusCode> {
    if session_id != *state.session_id {
        return Err(StatusCode::NOT_FOUND);
    }
    let safe_path = sanitize_relative_path(&file).ok_or(StatusCode::BAD_REQUEST)?;
    serve_file(&state.output_dir, safe_path.to_str().unwrap()).await
}

async fn serve_file(
    output_dir: &std::path::Path,
    file: &str,
) -> Result<Response<Body>, StatusCode> {
    let target_path = output_dir.join(file);
    let content = tokio::fs::read(&target_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", content_type_for_path(&target_path))
        .header("Cache-Control", "no-cache, no-store, must-revalidate")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from(content))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn sanitize_relative_path(path: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(path);
    if candidate.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(candidate)
}

fn content_type_for_path(path: &StdPath) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("m3u8") => "application/vnd.apple.mpegurl",
        Some("ts") => "video/mp2t",
        Some("mp4") => "video/mp4",
        _ => "application/octet-stream",
    }
}
