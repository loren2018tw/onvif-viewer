use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FFmpegStatus {
    pub installed: bool,
    pub install_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub onvif_port: u16,
    pub username: String,
    pub password: String,
    pub stream_uri: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredCamera {
    pub address: String,
    pub port: u16,
    pub name: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub xaddrs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRange {
    pub start_ip: String,
    pub end_ip: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct StreamInfo {
    pub uri: String,
    pub width: u32,
    pub height: u32,
}
