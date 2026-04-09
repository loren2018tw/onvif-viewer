use crate::models::CameraInfo;
use anyhow::{Context, Result};
use std::path::PathBuf;

fn storage_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Failed to find config directory")?;
    let app_dir = config_dir.join("onvif-viewer");
    std::fs::create_dir_all(&app_dir).context("Failed to create config directory")?;
    Ok(app_dir.join("cameras.json"))
}

pub fn load_cameras() -> Result<Vec<CameraInfo>> {
    let path = storage_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).context("Failed to read cameras file")?;
    let cameras: Vec<CameraInfo> =
        serde_json::from_str(&content).context("Failed to parse cameras file")?;
    Ok(cameras)
}

pub fn save_cameras(cameras: &[CameraInfo]) -> Result<()> {
    let path = storage_path()?;
    let content = serde_json::to_string_pretty(cameras).context("Failed to serialize cameras")?;
    std::fs::write(&path, content).context("Failed to write cameras file")?;
    Ok(())
}

pub fn add_camera(camera: CameraInfo) -> Result<Vec<CameraInfo>> {
    let mut cameras = load_cameras()?;
    // Check for duplicate by address+port
    cameras.retain(|c| !(c.address == camera.address && c.onvif_port == camera.onvif_port));
    cameras.push(camera);
    save_cameras(&cameras)?;
    Ok(cameras)
}

pub fn remove_camera(id: &str) -> Result<Vec<CameraInfo>> {
    let mut cameras = load_cameras()?;
    cameras.retain(|c| c.id != id);
    save_cameras(&cameras)?;
    Ok(cameras)
}

pub fn update_camera(camera: CameraInfo) -> Result<Vec<CameraInfo>> {
    let mut cameras = load_cameras()?;
    if let Some(existing) = cameras.iter_mut().find(|c| c.id == camera.id) {
        *existing = camera;
    }
    save_cameras(&cameras)?;
    Ok(cameras)
}
