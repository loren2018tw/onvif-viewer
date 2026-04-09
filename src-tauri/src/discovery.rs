use std::net::Ipv4Addr;
use std::time::Duration;
use anyhow::{Context, Result};
use crate::models::DiscoveredCamera;

/// Discover ONVIF cameras on the local network using WS-Discovery
pub async fn discover_cameras(duration_secs: u64) -> Result<Vec<DiscoveredCamera>> {
    let devices = oxvif::discovery::probe(Duration::from_secs(duration_secs)).await;

    let mut cameras = Vec::new();
    for device in devices {
        if let Some(xaddr) = device.xaddrs.first() {
            if let Ok(url) = url::Url::parse(xaddr) {
                let address = url.host_str().unwrap_or("unknown").to_string();
                let port = url.port().unwrap_or(80);

                // Extract name from scopes
                let name = device
                    .scopes
                    .iter()
                    .find(|s| s.contains("/name/"))
                    .and_then(|s| s.rsplit('/').next())
                    .map(|n| urlencoding::decode(n).unwrap_or_default().into_owned());

                cameras.push(DiscoveredCamera {
                    address,
                    port,
                    name,
                    manufacturer: None,
                    model: None,
                    xaddrs: device.xaddrs,
                });
            }
        }
    }

    Ok(cameras)
}

/// Scan a specific IP range for ONVIF cameras
pub async fn scan_range(
    start_ip: &str,
    end_ip: &str,
    port: u16,
) -> Result<Vec<DiscoveredCamera>> {
    let start: Ipv4Addr = start_ip.parse().context("Invalid start IP")?;
    let end: Ipv4Addr = end_ip.parse().context("Invalid end IP")?;

    let start_u32 = u32::from(start);
    let end_u32 = u32::from(end);

    if start_u32 > end_u32 {
        anyhow::bail!("Start IP must be less than or equal to end IP");
    }

    let mut cameras = Vec::new();
    let mut handles = Vec::new();

    for ip_u32 in start_u32..=end_u32 {
        let ip = Ipv4Addr::from(ip_u32);
        let addr = ip.to_string();
        let p = port;

        handles.push(tokio::spawn(async move {
            probe_camera(&addr, p).await
        }));
    }

    for handle in handles {
        if let Ok(Ok(Some(camera))) = handle.await {
            cameras.push(camera);
        }
    }

    Ok(cameras)
}

/// Probe a single IP:port for ONVIF service
async fn probe_camera(address: &str, port: u16) -> Result<Option<DiscoveredCamera>> {
    let url = format!("http://{}:{}/onvif/device_service", address, port);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()?;

    // Simple ONVIF GetSystemDateAndTime request to check if it's an ONVIF device
    let soap_body = r#"<?xml version="1.0" encoding="UTF-8"?>
    <s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
                xmlns:tds="http://www.onvif.org/ver10/device/wsdl">
        <s:Body>
            <tds:GetSystemDateAndTime/>
        </s:Body>
    </s:Envelope>"#;

    match client
        .post(&url)
        .header("Content-Type", "application/soap+xml; charset=utf-8")
        .body(soap_body)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            Ok(Some(DiscoveredCamera {
                address: address.to_string(),
                port,
                name: None,
                manufacturer: None,
                model: None,
                xaddrs: vec![url],
            }))
        }
        _ => Ok(None),
    }
}
