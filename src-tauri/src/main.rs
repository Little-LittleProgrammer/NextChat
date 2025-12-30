// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod fetch;
mod logger;
mod mcp;
mod stream;

fn main() {
    // 初始化日志系统
    let app_handle = tauri::Builder::default()
        .setup(|app| {
            // 获取应用数据目录
            let app_data_dir = app.path_resolver()
                .app_data_dir();
            
            // 初始化日志记录器
            if let Err(e) = logger::init_logger(app_data_dir) {
                eprintln!("Failed to initialize logger: {}", e);
            } else {
                log::info!("Logger initialized successfully");
            }
            
            Ok(())
        })
        .manage(mcp::McpState::new())
        .invoke_handler(tauri::generate_handler![
            stream::stream_fetch,
            fetch::http_fetch,
            fetch::http_fetch_text,
            fetch::http_fetch_json,
            mcp::mcp_read_config,
            mcp::mcp_write_config,
            mcp::mcp_read_user_config,
            mcp::mcp_import_user_config,
            mcp::mcp_start_server,
            mcp::mcp_stop_server,
            mcp::mcp_execute_command,
            mcp::mcp_get_server_status
        ])
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    
    log::info!("Starting NextChat Tauri application");
    app_handle.run(|_app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { .. } => {
                log::info!("Application exit requested");
            }
            _ => {}
        }
    });
}
