use crate::okx::TickerData;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSButton, NSColor, NSFont, NSFontAttributeName, NSForegroundColorAttributeName, NSBaselineOffsetAttributeName,
};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSDictionary, NSAttributedString, NSString};

/// Create the system tray icon with menu
pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    // Create menu items
    let toggle_dock_item = MenuItem::with_id(
        app,
        "toggle_dock",
        "隐藏 Dock 图标",
        true,
        None::<&str>,
    )?;

    let proxy_item = MenuItem::with_id(
        app,
        "proxy_config",
        "配置代理...",
        true,
        None::<&str>,
    )?;

    let quit_item = MenuItem::with_id(
        app,
        "quit",
        "退出应用",
        true,
        None::<&str>,
    )?;

    // Store toggle_dock_item reference before moving it
    let toggle_dock_item_ref = toggle_dock_item.clone();
    let proxy_item_ref = proxy_item.clone();

    // Create the menu
    let menu = Menu::with_items(app, &[&toggle_dock_item, &proxy_item, &quit_item])?;

    let tray = TrayIconBuilder::new()
        .icon(Image::from_bytes(include_bytes!("../icons/tray-neutral.png"))?)
        .tooltip("BTC Ticker - OKX BTC-USDT-SWAP")
        .title("---.-")
        .menu(&menu)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "toggle_dock" => {
                    toggle_dock_visibility(app);
                    // Update menu item text based on current state
                    update_menu_item_text(app);
                }
                "proxy_config" => {
                    show_proxy_config_dialog(app);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } => {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        // Position window near tray icon
                        position_window_near_tray(&window);
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            _ => {}
        })
        .show_menu_on_left_click(false)
        .build(app)?;

    // Store tray id and menu item references for later updates
    app.manage(TrayState::new(tray.id().clone(), toggle_dock_item_ref, proxy_item_ref));

    Ok(())
}

pub struct TrayState {
    pub tray_id: tauri::tray::TrayIconId,
    pub dock_visible: Mutex<bool>,
    pub toggle_dock_item: Arc<MenuItem<tauri::Wry>>,
    pub proxy_item: Arc<MenuItem<tauri::Wry>>,
    pub proxy_address: Mutex<String>,
}

impl TrayState {
    pub fn new(
        tray_id: tauri::tray::TrayIconId,
        toggle_dock_item: MenuItem<tauri::Wry>,
        proxy_item: MenuItem<tauri::Wry>,
    ) -> Self {
        Self {
            tray_id,
            dock_visible: Mutex::new(true), // Default to visible
            toggle_dock_item: Arc::new(toggle_dock_item),
            proxy_item: Arc::new(proxy_item),
            proxy_address: Mutex::new(String::new()), // Empty by default
        }
    }
}

/// Position the detail window near the tray icon area (top-right of screen)
fn position_window_near_tray(window: &tauri::WebviewWindow) {
    if let Ok(monitor) = window.primary_monitor() {
        if let Some(monitor) = monitor {
            let screen_size = monitor.size();
            let scale = monitor.scale_factor();
            let win_width = 400.0;
            let _win_height = 480.0;

            // Position at top-right, below the menu bar
            let x = (screen_size.width as f64 / scale) - win_width - 8.0;
            let y = 30.0; // Below macOS menu bar

            let _ = window.set_position(tauri::LogicalPosition::new(x, y));
        }
    }
}

/// Update the tray title with colored text using NSAttributedString on macOS
pub fn update_tray_title(app: &AppHandle, ticker: &TickerData) {
    let is_up = ticker.change >= 0.0;
    let price_text = format!("{:.1}", ticker.last);
    let pct_text = format!("{:.2}%", ticker.change_percent.abs());
    let arrow = if is_up { "↑" } else { "↓" };
    // Add extra spaces between price and arrow/percentage for better visual separation
    let full_title = format!("{}  {}{}", price_text, arrow, pct_text);

    // Try colored text on macOS, fall back to plain title
    #[cfg(target_os = "macos")]
    {
        if set_colored_title(app, &full_title, is_up) {
            return;
        }
    }

    // Fallback: plain text title
    if let Some(tray_state) = app.try_state::<TrayState>() {
        if let Some(tray) = app.tray_by_id(&tray_state.tray_id) {
            let _ = tray.set_title(Some(&full_title));
        }
    }
}

/// macOS-specific: set attributed (colored) title on the NSStatusItem button
#[cfg(target_os = "macos")]
fn set_colored_title(app: &AppHandle, title: &str, is_up: bool) -> bool {
    if let Some(tray_state) = app.try_state::<TrayState>() {
        if let Some(tray) = app.tray_by_id(&tray_state.tray_id) {
            let title = title.to_string();

            let attributed_title_applied = tray
                .with_inner_tray_icon(move |inner| {
                    let Some(ns_status_item) = inner.ns_status_item() else {
                        return false;
                    };

                    let Some(mtm) = MainThreadMarker::new() else {
                        return false;
                    };

                    let Some(button) = ns_status_item.button(mtm) else {
                        return false;
                    };
                    
                    // Configure button for better layout
                    button.setBordered(false);
                    
                    apply_colored_title(&button, &title, is_up)
                })
                .unwrap_or(false);

            // Use the newly generated Bitcoin symbol icons
            let icon_bytes: &[u8] = if is_up {
                include_bytes!("../icons/tray-up.png")
            } else {
                include_bytes!("../icons/tray-down.png")
            };

            if let Ok(icon) = Image::from_bytes(icon_bytes) {
                let _ = tray.set_icon(Some(icon));
            }

            return attributed_title_applied;
        }
    }

    false
}

#[cfg(target_os = "macos")]
fn apply_colored_title(button: &NSButton, title: &str, is_up: bool) -> bool {
    let ns_title = NSString::from_str(title);
    let color = if is_up {
        NSColor::systemGreenColor()
    } else {
        NSColor::systemRedColor()
    };
    // Use monospaced digit font to prevent width jumping when numbers change
    let font = NSFont::monospacedDigitSystemFontOfSize_weight(12.5, 0.0);
    
    // Adjust baseline offset to vertically align text with the Bitcoin icon
    // Negative value moves text down to align with icon
    let baseline_offset: f64 = -1.0;
    let baseline_value = objc2_foundation::NSNumber::numberWithDouble(baseline_offset);

    let attrs = unsafe {
        NSDictionary::<NSString, AnyObject>::from_slices(
            &[
                NSForegroundColorAttributeName, 
                NSFontAttributeName,
                NSBaselineOffsetAttributeName,
            ],
            &[color.as_ref(), font.as_ref(), baseline_value.as_ref()],
        )
    };

    let attributed_title = unsafe { NSAttributedString::new_with_attributes(&ns_title, &attrs) };
    button.setAttributedTitle(&attributed_title);
    
    true
}

/// Toggle dock visibility on macOS
fn toggle_dock_visibility(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::MainThreadMarker;
        use objc2_app_kit::NSApplication;
        
        if let Some(mtm) = MainThreadMarker::new() {
            if let Some(tray_state) = app.try_state::<TrayState>() {
                let mut dock_visible = tray_state.dock_visible.lock().unwrap();
                *dock_visible = !*dock_visible;
                let new_state = *dock_visible;
                
                // Use Objective-C to change activation policy
                let app_obj = NSApplication::sharedApplication(mtm);
                
                if new_state {
                    // Show in dock - use regular activation policy
                    app_obj.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Regular);
                    log::info!("Dock icon shown");
                } else {
                    // Hide from dock - use accessory activation policy
                    app_obj.setActivationPolicy(objc2_app_kit::NSApplicationActivationPolicy::Accessory);
                    log::info!("Dock icon hidden");
                }
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::info!("Dock visibility toggle not supported on this platform");
    }
}

/// Update menu item text based on current dock visibility state
fn update_menu_item_text(app: &AppHandle) {
    if let Some(tray_state) = app.try_state::<TrayState>() {
        let dock_visible = tray_state.dock_visible.lock().unwrap();
        let text = if *dock_visible {
            "隐藏 Dock 图标"
        } else {
            "显示 Dock 图标"
        };
        
        let _ = tray_state.toggle_dock_item.set_text(text);
    }
}

/// Show proxy configuration dialog
fn show_proxy_config_dialog(app: &AppHandle) {
    // Get current proxy address
    let current_proxy = if let Some(tray_state) = app.try_state::<TrayState>() {
        tray_state.proxy_address.lock().unwrap().clone()
    } else {
        String::new()
    };

    // Show input dialog using AppleScript on macOS
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        
        // Create helpful message with format examples
        let help_text = "代理服务器地址\n\n格式: 主机:端口\n示例: 127.0.0.1:7890\n     192.168.1.100:1080";
        
        let placeholder = if current_proxy.is_empty() {
            "127.0.0.1:7890"
        } else {
            &current_proxy
        };
        
        let script = format!(
            r#"
            display dialog "{0}" default answer "{1}" buttons {{"清除代理", "取消", "确定"}} default button "确定" cancel button "取消" with title "代理配置" with icon note
            text returned of result
            "#,
            help_text,
            placeholder
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    if let Ok(proxy_addr) = String::from_utf8(output.stdout) {
                        let proxy_addr = proxy_addr.trim().to_string();
                        if let Some(tray_state) = app.try_state::<TrayState>() {
                            let mut current_proxy = tray_state.proxy_address.lock().unwrap();
                            *current_proxy = proxy_addr.clone();
                            
                            // Update menu item text to show proxy status
                            let proxy_text = if proxy_addr.is_empty() {
                                "配置代理...".to_string()
                            } else {
                                format!("代理: {}", proxy_addr)
                            };
                            let _ = tray_state.proxy_item.set_text(&proxy_text);
                            
                            // Save proxy to file for persistence
                            save_proxy_config(&proxy_addr);
                            
                            // Notify the OKX module to restart with new proxy
                            restart_okx_connection(app, &proxy_addr);
                        }
                    }
                } else {
                    // Check if user clicked "清除代理" button
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if stderr.contains("清除代理") {
                        if let Some(tray_state) = app.try_state::<TrayState>() {
                            let mut current_proxy = tray_state.proxy_address.lock().unwrap();
                            *current_proxy = String::new();
                            
                            let _ = tray_state.proxy_item.set_text("配置代理...");
                            save_proxy_config("");
                            restart_okx_connection(app, "");
                            
                            log::info!("Proxy configuration cleared");
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to show proxy dialog: {}", e);
            }
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        log::info!("Proxy configuration not supported on this platform");
    }
}

/// Restart OKX connection with new proxy
fn restart_okx_connection(_app: &AppHandle, proxy_addr: &str) {
    // Update the proxy address in the OKX module
    crate::okx::set_proxy_address(proxy_addr.to_string());
    log::info!("Proxy configuration updated: {}", proxy_addr);
}

/// Save proxy configuration to file
fn save_proxy_config(proxy_addr: &str) {
    use std::fs;
    
    // Get app config directory
    if let Some(home_dir) = dirs::home_dir() {
        let config_dir = home_dir.join(".config").join("btc-ticker");
        let config_file = config_dir.join("proxy.conf");
        
        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&config_dir) {
            log::error!("Failed to create config directory: {}", e);
            return;
        }
        
        // Write proxy address to file
        if let Err(e) = fs::write(&config_file, proxy_addr) {
            log::error!("Failed to save proxy config: {}", e);
        } else {
            log::info!("Proxy config saved to: {:?}", config_file);
        }
    }
}

/// Load proxy configuration from file
pub fn load_proxy_config() -> String {
    use std::fs;
    
    if let Some(home_dir) = dirs::home_dir() {
        let config_file = home_dir.join(".config").join("btc-ticker").join("proxy.conf");
        
        if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(proxy_addr) => {
                    log::info!("Loaded proxy config: {}", proxy_addr);
                    return proxy_addr.trim().to_string();
                }
                Err(e) => {
                    log::error!("Failed to read proxy config: {}", e);
                }
            }
        }
    }
    
    String::new()
}
