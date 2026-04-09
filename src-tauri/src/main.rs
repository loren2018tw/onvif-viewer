// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let stream_manager = std::sync::Arc::new(tokio::sync::Mutex::new(
        onvif_viewer_lib::StreamManager::new(),
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(stream_manager)
        .invoke_handler(tauri::generate_handler![
            onvif_viewer_lib::discover_onvif_cameras,
            onvif_viewer_lib::scan_onvif_range,
            onvif_viewer_lib::get_stream_uri,
            onvif_viewer_lib::list_cameras,
            onvif_viewer_lib::save_camera,
            onvif_viewer_lib::delete_camera,
            onvif_viewer_lib::update_camera_info,
            onvif_viewer_lib::test_camera_connection,
            onvif_viewer_lib::diagnose_camera,
            onvif_viewer_lib::start_preview,
            onvif_viewer_lib::stop_preview,
            onvif_viewer_lib::check_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
