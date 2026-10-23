use crate::models::CameraInfo;
use anyhow::{Context, Result};
use std::cmp::Ordering;
use std::net::Ipv4Addr;
use std::path::PathBuf;

fn storage_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Failed to find config directory")?;
    let app_dir = config_dir.join("onvif-viewer");
    std::fs::create_dir_all(&app_dir).context("Failed to create config directory")?;
    Ok(app_dir.join("cameras.json"))
}

/// 依 IPv4 數值排序，同 IP 再依連接埠升冪；無法解析為 IPv4 的位址排在最後。
fn sort_cameras(cameras: &mut [CameraInfo]) {
    cameras.sort_by(|a, b| {
        match (
            a.address.parse::<Ipv4Addr>(),
            b.address.parse::<Ipv4Addr>(),
        ) {
            (Ok(a_ip), Ok(b_ip)) => a_ip
                .cmp(&b_ip)
                .then_with(|| a.onvif_port.cmp(&b.onvif_port)),
            (Ok(_), Err(_)) => Ordering::Less,
            (Err(_), Ok(_)) => Ordering::Greater,
            (Err(_), Err(_)) => a
                .address
                .to_lowercase()
                .cmp(&b.address.to_lowercase())
                .then_with(|| a.onvif_port.cmp(&b.onvif_port)),
        }
    });
}

pub fn load_cameras() -> Result<Vec<CameraInfo>> {
    let path = storage_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path).context("Failed to read cameras file")?;
    let mut cameras: Vec<CameraInfo> =
        serde_json::from_str(&content).context("Failed to parse cameras file")?;
    sort_cameras(&mut cameras);
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
    sort_cameras(&mut cameras);
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
    sort_cameras(&mut cameras);
    save_cameras(&cameras)?;
    Ok(cameras)
}

#[cfg(test)]
mod tests {
    use super::sort_cameras;
    use crate::models::CameraInfo;
    use chrono::Utc;

    fn camera(name: &str, address: &str, port: u16) -> CameraInfo {
        CameraInfo {
            id: format!("{address}:{port}"),
            name: name.to_string(),
            address: address.to_string(),
            onvif_port: port,
            username: "admin".to_string(),
            password: String::new(),
            stream_uri: None,
            manufacturer: None,
            model: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn sorts_by_numeric_ipv4_then_port() {
        // 名稱刻意與 IP 順序相反，確保排序鍵是 address 而不是 name。
        let mut cameras = vec![
            camera("Z 前門", "10.1.6.100", 80),
            camera("A 後院", "10.1.6.99", 80),
            camera("M 車庫", "10.1.6.9", 80),
            camera("B 頂樓", "10.1.6.10", 80),
            camera("C 一樓", "10.1.6.1", 8000),
            camera("D 二樓", "10.1.6.1", 80),
            camera("E 主機", "camera.local", 80),
        ];

        sort_cameras(&mut cameras);

        let order: Vec<String> = cameras
            .iter()
            .map(|c| format!("{}:{}", c.address, c.onvif_port))
            .collect();
        assert_eq!(
            order,
            vec![
                "10.1.6.1:80",
                "10.1.6.1:8000",
                "10.1.6.9:80",
                "10.1.6.10:80",
                "10.1.6.99:80",
                "10.1.6.100:80",
                "camera.local:80",
            ]
        );
    }
}
