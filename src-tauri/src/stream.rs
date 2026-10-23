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
    time::{sleep, timeout, Duration, Instant},
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

struct ActiveStream {
    output_dir: PathBuf,
    snapshot_path: Option<PathBuf>,
    ffmpeg: Child,
    shutdown_tx: oneshot::Sender<()>,
}

fn snapshot_dir() -> PathBuf {
    std::env::temp_dir().join("onvif-viewer-snapshots")
}

/// Path for the per-camera last-frame JPEG. Camera ids are UUIDs, but validate
/// anyway so the frontend can never escape the snapshot directory.
pub fn camera_snapshot_path(camera_id: &str) -> Option<PathBuf> {
    if camera_id.is_empty()
        || !camera_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }

    let dir = snapshot_dir();
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join(format!("{camera_id}.jpg")))
}

pub fn delete_camera_snapshot(camera_id: &str) {
    if let Some(path) = camera_snapshot_path(camera_id) {
        let _ = std::fs::remove_file(path);
    }
}

/// Remove snapshots from the previous app run. Last frames are only meant to
/// survive while the application is open.
pub fn clear_stale_snapshots() {
    let _ = std::fs::remove_dir_all(snapshot_dir());
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
    active_generation: u64,
    ffmpeg_checked: bool,
}

#[derive(Clone)]
struct PreviewState {
    output_dir: Arc<PathBuf>,
    session_id: Arc<String>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            active: None,
            active_generation: 0,
            ffmpeg_checked: false,
        }
    }

    pub async fn start(
        &mut self,
        rtsp_uri: &str,
        has_audio_hint: Option<bool>,
        camera_id: Option<String>,
        generation: u64,
    ) -> Result<PreviewSession> {
        self.stop().await;

        let snapshot_path = camera_id.as_deref().and_then(camera_snapshot_path);

        // Verify ffmpeg is available once per app run.
        if !self.ffmpeg_checked {
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
            self.ffmpeg_checked = true;
        }

        let session_id = uuid::Uuid::new_v4().to_string();
        let output_dir = std::env::temp_dir()
            .join("onvif-viewer-preview")
            .join(&session_id);
        std::fs::create_dir_all(&output_dir).context("無法建立預覽輸出目錄")?;
        let playlist_path = output_dir.join("index.m3u8");
        let segment_pattern = output_dir.join("segment_%03d.ts");
        let ffmpeg_log_path = output_dir.join("ffmpeg.log");
        let has_audio = match has_audio_hint {
            Some(has_audio) => has_audio,
            None => probe_has_audio(rtsp_uri).await,
        };

        // Spawn FFmpeg: RTSP → HLS (video + optional audio).
        // Try low-latency probing first (SDP already contains codec info), and
        // fall back to normal probing if a camera needs more input data.
        let mut active_ffmpeg = None;
        let mut last_error: Option<(anyhow::Error, Option<String>)> = None;
        for (attempt, low_latency) in [true, false].into_iter().enumerate() {
            let _ = tokio::fs::remove_file(&playlist_path).await;
            let mut ffmpeg = spawn_hls_ffmpeg(
                rtsp_uri,
                low_latency,
                &segment_pattern,
                &playlist_path,
                &ffmpeg_log_path,
            )?;

            match wait_for_playlist(&mut ffmpeg, &playlist_path).await {
                Ok(()) => {
                    active_ffmpeg = Some(ffmpeg);
                    break;
                }
                Err(err) => {
                    let _ = ffmpeg.kill().await;
                    let details = read_ffmpeg_log(&ffmpeg_log_path).await;
                    let should_retry =
                        attempt == 0 && should_retry_with_default_probe(details.as_deref());
                    last_error = Some((err, details));
                    if !should_retry {
                        break;
                    }
                }
            }
        }

        let ffmpeg = match active_ffmpeg {
            Some(ffmpeg) => ffmpeg,
            None => {
                let _ = std::fs::remove_dir_all(&output_dir);
                let (err, details) =
                    last_error.expect("ffmpeg startup attempt always records an error");
                return Err(match details {
                    Some(details) => err.context(format!("ffmpeg 輸出：{details}")),
                    None => err,
                });
            }
        };

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
            snapshot_path,
            ffmpeg,
            shutdown_tx,
        });
        self.active_generation = generation;

        Ok(PreviewSession {
            session_id,
            preview_url,
            stream_uri: rtsp_uri.to_string(),
            has_audio,
        })
    }

    pub async fn stop(&mut self) {
        self.active_generation = 0;
        if let Some(mut active) = self.active.take() {
            let _ = active.shutdown_tx.send(());
            let _ = active.ffmpeg.kill().await;
            if let Some(path) = &active.snapshot_path {
                write_snapshot_from_hls(&active.output_dir, path);
            }
            let _ = std::fs::remove_dir_all(&active.output_dir);
        }
    }

    /// Synchronous variant used when the process is exiting and awaiting the
    /// async child cleanup is no longer possible.
    pub fn stop_sync(&mut self) {
        self.active_generation = 0;
        if let Some(mut active) = self.active.take() {
            let _ = active.shutdown_tx.send(());
            let _ = active.ffmpeg.start_kill();
            let _ = active.ffmpeg.try_wait();
            if let Some(path) = &active.snapshot_path {
                write_snapshot_from_hls(&active.output_dir, path);
            }
            let _ = std::fs::remove_dir_all(&active.output_dir);
        }
    }

    /// Stop only if the active stream was started before `generation`.
    /// A newer preview may have already replaced it while `stop_preview` was
    /// waiting for the manager lock.
    pub async fn stop_if_older_than(&mut self, generation: u64) {
        if self.active_generation <= generation {
            self.stop().await;
        }
    }
}

impl Drop for StreamManager {
    fn drop(&mut self) {
        // Normal app shutdown may not run async cleanup; make sure an active
        // preview does not keep pulling the camera stream after the window closes.
        self.stop_sync();
    }
}

fn spawn_hls_ffmpeg(
    rtsp_uri: &str,
    low_latency: bool,
    segment_pattern: &StdPath,
    playlist_path: &StdPath,
    log_path: &StdPath,
) -> Result<Child> {
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg.args(["-hide_banner", "-loglevel", "error"]);

    if low_latency {
        // RTSP SDP already carries the codec information, so waiting for a
        // large probe buffer only delays the first segment. Some cameras do
        // need normal probing; start() retries with this block disabled.
        ffmpeg.args([
            "-fflags",
            "nobuffer",
            "-flags",
            "low_delay",
            "-probesize",
            "32",
            "-analyzeduration",
            "0",
        ]);
    }

    ffmpeg
        .args([
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
        .stderr(Stdio::from(
            std::fs::File::create(log_path).context("無法建立預覽記錄檔")?,
        ))
        .kill_on_drop(true);
    hide_window(&mut ffmpeg);
    ffmpeg.spawn().context("無法啟動 ffmpeg")
}

fn snapshot_segments(output_dir: &StdPath) -> Vec<PathBuf> {
    let mut names: Vec<String> = Vec::new();
    if let Ok(content) = std::fs::read_to_string(output_dir.join("index.m3u8")) {
        for line in content.lines().rev() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.ends_with(".ts") {
                names.push(line.to_string());
                if names.len() >= 2 {
                    break;
                }
            }
        }
    }

    if names.is_empty() {
        if let Ok(entries) = std::fs::read_dir(output_dir) {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("ts"))
                .collect();
            paths.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
            paths.truncate(2);
            return paths;
        }
    }

    names
        .into_iter()
        .map(|name| output_dir.join(name))
        .filter(|path| path.exists())
        .collect()
}

/// Extract a JPEG from the newest complete HLS segment. WebKitGTK returns a
/// black frame when drawing an MSE-backed video onto a canvas, so the last
/// frame must be produced by ffmpeg instead of the webview.
fn write_snapshot_from_hls(output_dir: &StdPath, snapshot_path: &StdPath) {
    let segments = snapshot_segments(output_dir);
    if segments.is_empty() {
        return;
    }

    let temp_path = snapshot_path.with_extension("tmp.jpg");
    for segment in segments {
        let mut cmd = std::process::Command::new("ffmpeg");
        cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
            .arg(&segment)
            .args([
                "-frames:v",
                "1",
                "-vf",
                "scale=640:-2",
                "-q:v",
                "4",
                "-an",
                "-f",
                "image2",
            ])
            .arg(&temp_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        if cmd.status().map(|status| status.success()).unwrap_or(false) {
            let _ = std::fs::remove_file(snapshot_path);
            let _ = std::fs::rename(&temp_path, snapshot_path);
            return;
        }
        let _ = std::fs::remove_file(&temp_path);
    }
}

fn should_retry_with_default_probe(details: Option<&str>) -> bool {
    let Some(details) = details else {
        return false;
    };
    let details = details.to_lowercase();

    let looks_like_network_error = [
        "no route to host",
        "connection refused",
        "connection reset",
        "network is unreachable",
        "timed out",
        "unauthorized",
        "401",
        "403",
        "404",
    ]
    .iter()
    .any(|needle| details.contains(needle));
    if looks_like_network_error {
        return false;
    }

    ["probe", "codec", "invalid data", "unspecified", "could not find"]
        .iter()
        .any(|needle| details.contains(needle))
}

async fn probe_has_audio(rtsp_uri: &str) -> bool {
    let mut ffprobe = Command::new("ffprobe");
    ffprobe
        .args([
            "-v",
            "error",
            "-fflags",
            "nobuffer",
            "-flags",
            "low_delay",
            "-probesize",
            "32",
            "-analyzeduration",
            "0",
            "-rtsp_transport",
            "tcp",
            "-select_streams",
            "a",
            "-show_entries",
            "stream=index",
            "-of",
            "csv=p=0",
            rtsp_uri,
        ])
        .kill_on_drop(true);
    hide_window(&mut ffprobe);

    // 音訊偵測只是附加功能，不能讓它卡住整個預覽流程。
    match timeout(Duration::from_secs(5), ffprobe.output()).await {
        Ok(Ok(output)) if output.status.success() => {
            !String::from_utf8_lossy(&output.stdout).trim().is_empty()
        }
        Ok(Err(err)) => {
            eprintln!("ffprobe 音訊偵測失敗，略過音訊：{err}");
            false
        }
        Err(_) => {
            eprintln!("ffprobe 音訊偵測逾時，略過音訊");
            false
        }
        _ => false,
    }
}

async fn read_ffmpeg_log(log_path: &StdPath) -> Option<String> {
    let content = tokio::fs::read_to_string(log_path).await.ok()?;
    let lines: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    if lines.is_empty() {
        return None;
    }

    let start = lines.len().saturating_sub(5);
    Some(lines[start..].join(" | "))
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

#[cfg(test)]
mod tests {
    use super::{camera_snapshot_path, should_retry_with_default_probe};

    #[test]
    fn retries_only_when_probing_may_be_the_cause() {
        assert!(should_retry_with_default_probe(Some(
            "Could not find codec parameters for stream 0"
        )));
        assert!(should_retry_with_default_probe(Some(
            "Invalid data found when processing input"
        )));
        assert!(!should_retry_with_default_probe(Some(
            "Connection refused"
        )));
        assert!(!should_retry_with_default_probe(Some(
            "No route to host"
        )));
        assert!(!should_retry_with_default_probe(None));
    }

    #[test]
    fn snapshot_path_rejects_unsafe_camera_ids() {
        assert!(camera_snapshot_path("").is_none());
        assert!(camera_snapshot_path("../other-camera").is_none());
        assert!(camera_snapshot_path("a/b").is_none());
        assert!(camera_snapshot_path("camera-id_123").is_some());
    }
}
