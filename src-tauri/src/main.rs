// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    // Last-frame snapshots only live for the current app run.
    onvif_viewer_lib::stream::clear_stale_snapshots();

    let stream_manager = std::sync::Arc::new(tokio::sync::Mutex::new(
        onvif_viewer_lib::StreamManager::new(),
    ));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(stream_manager)
        .manage(onvif_viewer_lib::PreviewCancellation::default())
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
            onvif_viewer_lib::get_camera_snapshot,
            onvif_viewer_lib::check_ffmpeg,
            onvif_viewer_lib::get_app_version,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                let state = app_handle.state::<
                    std::sync::Arc<tokio::sync::Mutex<onvif_viewer_lib::StreamManager>>,
                >();
                let mut manager = state.blocking_lock();
                manager.stop_sync();
            }
        });
}
