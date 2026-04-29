mod okx;
mod tray;

use log::info;
use okx::TickerData;
use tauri::{Emitter, Manager};
use tokio::sync::watch;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Initialize proxy storage first
            okx::init_proxy_storage();
            
            // Load saved proxy configuration
            let saved_proxy = tray::load_proxy_config();
            if !saved_proxy.is_empty() {
                okx::set_proxy_address(saved_proxy.clone());
                info!("Loaded proxy config: {}", saved_proxy);
            }
            
            // Create the system tray
            tray::create_tray(app.handle())?;
            
            // Update proxy menu item text if proxy is configured
            if !saved_proxy.is_empty() {
                if let Some(tray_state) = app.handle().try_state::<tray::TrayState>() {
                    let proxy_text = format!("代理: {}", saved_proxy);
                    let _ = tray_state.proxy_item.set_text(&proxy_text);
                    let mut proxy_addr = tray_state.proxy_address.lock().unwrap();
                    *proxy_addr = saved_proxy;
                }
            }

            // Create a watch channel for price updates
            let (tx, mut rx) = watch::channel(TickerData::default());

            // Start the OKX WebSocket client
            okx::start_ws_client(tx);

            // Spawn a task to react to price changes
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if rx.changed().await.is_ok() {
                        let ticker = rx.borrow().clone();

                        // Update tray title with price
                        tray::update_tray_title(&app_handle, &ticker);

                        // Emit event to frontend
                        let _ = app_handle.emit("price-update", &ticker);
                    }
                }
            });

            info!("BTC Ticker app started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
