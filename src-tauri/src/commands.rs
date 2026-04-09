use crate::models::{CameraInfo, DiscoveredCamera, FFmpegStatus, ScanRange};
use crate::stream::StreamManager;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[tauri::command]
pub async fn discover_onvif_cameras(
    duration_secs: Option<u64>,
) -> Result<Vec<DiscoveredCamera>, String> {
    let duration = duration_secs.unwrap_or(5);
    crate::discovery::discover_cameras(duration)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn scan_onvif_range(range: ScanRange) -> Result<Vec<DiscoveredCamera>, String> {
    crate::discovery::scan_range(&range.start_ip, &range.end_ip, range.port)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stream_uri(
    address: String,
    port: u16,
    username: String,
    password: String,
    stream_type: Option<String>,
) -> Result<String, String> {
    crate::camera::get_stream_uri(
        &address,
        port,
        &username,
        &password,
        stream_type.as_deref().unwrap_or("sub"),
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_camera_connection(
    address: String,
    port: u16,
    username: String,
    password: String,
) -> Result<(), String> {
    crate::camera::test_connection(&address, port, &username, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn diagnose_camera(
    address: String,
    port: u16,
    username: String,
    password: String,
) -> Result<String, String> {
    crate::camera::diagnose_camera(&address, port, &username, &password)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_cameras() -> Result<Vec<CameraInfo>, String> {
    crate::storage::load_cameras().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_camera(
    name: String,
    address: String,
    onvif_port: u16,
    username: String,
    password: String,
    stream_uri: Option<String>,
    manufacturer: Option<String>,
    model: Option<String>,
) -> Result<Vec<CameraInfo>, String> {
    let now = Utc::now();
    let camera = CameraInfo {
        id: Uuid::new_v4().to_string(),
        name,
        address,
        onvif_port,
        username,
        password,
        stream_uri,
        manufacturer,
        model,
        created_at: now,
        updated_at: now,
    };
    crate::storage::add_camera(camera).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_camera(id: String) -> Result<Vec<CameraInfo>, String> {
    crate::storage::remove_camera(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_camera_info(camera: CameraInfo) -> Result<Vec<CameraInfo>, String> {
    let cameras = crate::storage::load_cameras().map_err(|e| e.to_string())?;
    let mut updated = camera;
    // Preserve created_at and optional fields from the existing record
    if let Some(existing) = cameras.iter().find(|c| c.id == updated.id) {
        updated.created_at = existing.created_at;
        if updated.stream_uri.is_none() {
            updated.stream_uri = existing.stream_uri.clone();
        }
        if updated.manufacturer.is_none() {
            updated.manufacturer = existing.manufacturer.clone();
        }
        if updated.model.is_none() {
            updated.model = existing.model.clone();
        }
    }
    updated.updated_at = Utc::now();
    crate::storage::update_camera(updated).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_preview(
    address: String,
    port: u16,
    username: String,
    password: String,
    stream_type: Option<String>,
    state: tauri::State<'_, Arc<Mutex<StreamManager>>>,
) -> Result<String, String> {
    // First get the RTSP stream URI
    let rtsp_uri = crate::camera::get_stream_uri(
        &address,
        port,
        &username,
        &password,
        stream_type.as_deref().unwrap_or("sub"),
    )
    .await
    .map_err(|e| e.to_string())?;

    let mut manager = state.lock().await;
    manager.start(&rtsp_uri).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_preview(
    state: tauri::State<'_, Arc<Mutex<StreamManager>>>,
) -> Result<(), String> {
    let mut manager = state.lock().await;
    manager.stop().await;
    Ok(())
}

#[tauri::command]
pub fn check_ffmpeg() -> Result<FFmpegStatus, String> {
    use std::process::Command;

    let os = std::env::consts::OS;

    // Try to run ffmpeg -version to check if it's installed
    let output = Command::new("ffmpeg").arg("-version").output();

    match output {
        Ok(output) if output.status.success() => Ok(FFmpegStatus {
            installed: true,
            install_command: String::new(),
        }),
        _ => {
            let install_command = match os {
                "windows" => "winget install Gyan.FFmpeg".to_string(),
                "linux" => "apt install ffmpeg".to_string(),
                "macos" => "brew install ffmpeg".to_string(),
                _ => "Please install FFmpeg from https://ffmpeg.org/download.html".to_string(),
            };

            Ok(FFmpegStatus {
                installed: false,
                install_command,
            })
        }
    }
}
