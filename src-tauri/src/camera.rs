use anyhow::{Context, Result};
use oxvif::OnvifSession;

/// Check if a URI looks like a valid RTSP stream URL
fn is_valid_rtsp_uri(uri: &str) -> bool {
    uri.starts_with("rtsp://") || uri.starts_with("rtsps://")
}

/// Embed credentials into an RTSP URI if not already present
fn embed_credentials(uri: &str, username: &str, password: &str) -> String {
    if let Ok(mut parsed) = url::Url::parse(uri) {
        if parsed.username().is_empty() && !username.is_empty() {
            let _ = parsed.set_username(username);
            let _ = parsed.set_password(Some(password));
            return parsed.to_string();
        }
    }
    uri.to_string()
}

/// Extract RTSP URI from raw SOAP GetStreamUriResponse XML.
/// Handles cases where the XML parser mishandles &amp; entities.
fn extract_uri_from_soap_xml(xml: &str) -> Option<String> {
    // Find <tt:Uri>...</tt:Uri> or <Uri>...</Uri>
    let uri_start = xml.find("<tt:Uri>").map(|i| i + 8)
        .or_else(|| xml.find("<Uri>").map(|i| i + 5))?;
    let uri_end = xml[uri_start..].find("</tt:Uri>")
        .or_else(|| xml[uri_start..].find("</Uri>"))?;
    let raw = &xml[uri_start..uri_start + uri_end];
    // Decode XML entities
    let decoded = raw
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    if is_valid_rtsp_uri(&decoded) {
        Some(decoded)
    } else {
        None
    }
}

/// Get RTSP stream URI from an ONVIF camera using oxvif
/// Iterates all profiles and tries both Media1 and Media2 for each.
/// Falls back to raw SOAP parsing when oxvif misparses URIs with &amp; entities.
pub async fn get_stream_uri(
    address: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<String> {
    let device_url = format!("http://{}:{}/onvif/device_service", address, port);

    let session = OnvifSession::builder(&device_url)
        .with_credentials(username, password)
        .with_clock_sync()
        .build()
        .await
        .context("Failed to connect to ONVIF device")?;

    let profiles = session
        .get_profiles()
        .await
        .context("Failed to get media profiles")?;

    if profiles.is_empty() {
        anyhow::bail!("No media profiles available");
    }

    let mut last_raw_uri = String::new();

    // Try each profile with Media1, then Media2
    for profile in &profiles {
        // Try Media1
        if let Ok(stream_uri) = session.get_stream_uri(&profile.token).await {
            if is_valid_rtsp_uri(&stream_uri.uri) {
                return Ok(embed_credentials(&stream_uri.uri, username, password));
            }
            last_raw_uri = stream_uri.uri.clone();
        }

        // Try Media2
        if let Ok(uri) = session.get_stream_uri_media2(&profile.token).await {
            if is_valid_rtsp_uri(&uri) {
                return Ok(embed_credentials(&uri, username, password));
            }
            if last_raw_uri.is_empty() {
                last_raw_uri = uri;
            }
        }
    }

    // Fallback: oxvif may misparse URIs with &amp; entities (e.g. Dahua cameras).
    // Try raw SOAP request and extract URI ourselves.
    if let Some(media_url) = &session.capabilities().media.url {
        for profile in &profiles {
            if let Ok(raw_xml) = raw_get_stream_uri(media_url, &profile.token, username, password).await {
                if let Some(uri) = extract_uri_from_soap_xml(&raw_xml) {
                    return Ok(embed_credentials(&uri, username, password));
                }
            }
        }
    }

    anyhow::bail!(
        "所有 profile 皆無法取得有效的 RTSP URI（裝置回傳：{}）",
        last_raw_uri
    )
}

/// Test connection to an ONVIF camera — verifies auth by requesting device info
pub async fn test_connection(
    address: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<()> {
    let device_url = format!("http://{}:{}/onvif/device_service", address, port);

    let session = OnvifSession::builder(&device_url)
        .with_credentials(username, password)
        .with_clock_sync()
        .build()
        .await
        .context("無法連線至 ONVIF 裝置")?;

    // GetDeviceInformation requires authentication on most cameras,
    // unlike GetProfiles which some cameras allow without auth.
    session
        .client()
        .get_device_info()
        .await
        .context("認證失敗，請確認帳號密碼是否正確")?;

    Ok(())
}

/// Diagnose an ONVIF camera — return detailed info for debugging
pub async fn diagnose_camera(
    address: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<String> {
    let device_url = format!("http://{}:{}/onvif/device_service", address, port);
    let mut report = format!("=== ONVIF 診斷報告 ===\n裝置: {}:{}\n\n", address, port);

    // 1. Build session
    let session = match OnvifSession::builder(&device_url)
        .with_credentials(username, password)
        .with_clock_sync()
        .build()
        .await
    {
        Ok(s) => s,
        Err(e) => {
            report.push_str(&format!("❌ 無法建立連線: {}\n", e));
            return Ok(report);
        }
    };
    report.push_str("✅ ONVIF 連線成功\n\n");

    // 2. Capabilities
    let caps = session.capabilities();
    report.push_str("【服務端點】\n");
    report.push_str(&format!("  Device:  {:?}\n", caps.device.url));
    report.push_str(&format!("  Media:   {:?}\n", caps.media.url));
    report.push_str(&format!("  Media2:  {:?}\n", caps.media2.url));
    report.push_str(&format!("  PTZ:     {:?}\n", caps.ptz.url));
    report.push('\n');

    // 3. Device info
    match session.client().get_device_info().await {
        Ok(info) => {
            report.push_str("【裝置資訊】（需要認證）\n");
            report.push_str(&format!("  製造商:  {}\n", info.manufacturer));
            report.push_str(&format!("  型號:    {}\n", info.model));
            report.push_str(&format!("  韌體:    {}\n", info.firmware_version));
            report.push_str(&format!("  序號:    {}\n", info.serial_number));
            report.push_str("  ✅ 帳號密碼驗證通過\n\n");
        }
        Err(e) => {
            report.push_str(&format!("❌ 取得裝置資訊失敗（帳密可能有誤）: {}\n\n", e));
        }
    }

    // 4. Profiles + raw SOAP for first profile
    match session.get_profiles().await {
        Ok(profiles) => {
            report.push_str(&format!("【媒體 Profile】（共 {} 個）\n", profiles.len()));
            for (i, p) in profiles.iter().enumerate() {
                report.push_str(&format!(
                    "  #{} name=\"{}\" token=\"{}\" video_enc={:?}\n",
                    i + 1,
                    p.name,
                    p.token,
                    p.video_encoder_token
                ));

                // Try Media1 GetStreamUri
                match session.get_stream_uri(&p.token).await {
                    Ok(su) => {
                        let valid = if is_valid_rtsp_uri(&su.uri) { "✅" } else { "⚠️" };
                        report.push_str(&format!("    Media1 URI: {} {}\n", valid, su.uri));
                    }
                    Err(e) => {
                        report.push_str(&format!("    Media1 URI: ❌ {}\n", e));
                    }
                }

                // Try Media2 GetStreamUri
                match session.get_stream_uri_media2(&p.token).await {
                    Ok(uri) => {
                        let valid = if is_valid_rtsp_uri(&uri) { "✅" } else { "⚠️" };
                        report.push_str(&format!("    Media2 URI: {} {}\n", valid, uri));
                    }
                    Err(e) => {
                        report.push_str(&format!("    Media2 URI: ❌ {}\n", e));
                    }
                }
            }

            // 5. Raw SOAP request for the first profile to see exact XML response
            if let Some(first_profile) = profiles.first() {
                if let Some(media_url) = &caps.media.url {
                    report.push_str("\n【原始 SOAP 回應（第一個 Profile）】\n");
                    match raw_get_stream_uri(media_url, &first_profile.token, username, password).await {
                        Ok(raw_xml) => {
                            report.push_str(&raw_xml);
                            report.push('\n');
                        }
                        Err(e) => {
                            report.push_str(&format!("❌ 原始請求失敗: {}\n", e));
                        }
                    }
                }
            }
        }
        Err(e) => {
            report.push_str(&format!("❌ 取得 Profile 失敗: {}\n", e));
        }
    }

    Ok(report)
}

/// Send a raw SOAP GetStreamUri request and return the raw XML response
async fn raw_get_stream_uri(
    media_url: &str,
    profile_token: &str,
    _username: &str,
    _password: &str,
) -> Result<String> {
    let soap_body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
            xmlns:trt="http://www.onvif.org/ver10/media/wsdl"
            xmlns:tt="http://www.onvif.org/ver10/schema">
  <s:Body>
    <trt:GetStreamUri>
      <trt:StreamSetup>
        <tt:Stream>RTP-Unicast</tt:Stream>
        <tt:Transport><tt:Protocol>RTSP</tt:Protocol></tt:Transport>
      </trt:StreamSetup>
      <trt:ProfileToken>{profile_token}</trt:ProfileToken>
    </trt:GetStreamUri>
  </s:Body>
</s:Envelope>"#,
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(media_url)
        .header("Content-Type", "application/soap+xml; charset=utf-8; action=\"http://www.onvif.org/ver10/media/wsdl/GetStreamUri\"")
        .body(soap_body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .context("HTTP 請求失敗")?;

    let status = resp.status();
    let body = resp.text().await.context("讀取回應失敗")?;

    Ok(format!("HTTP {}\n{}", status, body))
}

/// Get device info (manufacturer, model) from an ONVIF camera
#[allow(dead_code)]
pub async fn get_device_info(
    address: &str,
    port: u16,
    username: &str,
    password: &str,
) -> Result<(String, String)> {
    let device_url = format!("http://{}:{}/onvif/device_service", address, port);

    let session = OnvifSession::builder(&device_url)
        .with_credentials(username, password)
        .with_clock_sync()
        .build()
        .await
        .context("Failed to connect to ONVIF device")?;

    let info = session
        .client()
        .get_device_info()
        .await
        .context("Failed to get device info")?;

    Ok((info.manufacturer, info.model))
}
