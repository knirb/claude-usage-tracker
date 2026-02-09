mod keychain;
mod usage_api;

use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};
use usage_api::UsageData;

struct AppState {
    last_usage: Option<UsageData>,
    http_client: reqwest::Client,
}

/// Read fresh tokens from keychain, fetch usage, handle 401 with token refresh.
async fn fetch_fresh_usage(client: &reqwest::Client) -> Result<UsageData, String> {
    let tokens = keychain::read_keychain()?;

    match usage_api::fetch_usage(client, &tokens.access_token).await {
        Ok(data) => Ok(data),
        Err(e) if e.contains("401") => {
            // Token expired — try refresh then retry
            let new_token =
                keychain::refresh_access_token(client, &tokens.refresh_token).await?;
            usage_api::fetch_usage(client, &new_token).await
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
async fn get_usage(state: tauri::State<'_, Mutex<AppState>>) -> Result<UsageData, String> {
    let client = {
        state.lock().unwrap().http_client.clone()
    };

    let data = fetch_fresh_usage(&client).await?;

    {
        let mut s = state.lock().unwrap();
        s.last_usage = Some(data.clone());
    }
    Ok(data)
}

#[tauri::command]
async fn get_cached_usage(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Option<UsageData>, String> {
    let s = state.lock().unwrap();
    Ok(s.last_usage.clone())
}

pub fn run() {
    let state = AppState {
        last_usage: None,
        http_client: reqwest::Client::new(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .manage(Mutex::new(state))
        .invoke_handler(tauri::generate_handler![get_usage, get_cached_usage])
        .setup(|app| {
            // Hide from Dock — menubar-only app
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Build tray menu (right-click)
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            // Load tray icon
            let icon = Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                .expect("Failed to load tray icon");

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id() == "quit" {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    if let tauri::tray::TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_popup(app);
                    }
                })
                .build(app)?;

            // Start polling timer
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                interval.tick().await; // skip first immediate tick
                loop {
                    interval.tick().await;
                    poll_usage(&app_handle).await;
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {});
}

fn toggle_popup(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("popup") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.move_window(Position::TrayCenter);
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        // Create popup window on first click
        let window = WebviewWindowBuilder::new(app, "popup", WebviewUrl::default())
            .title("Claude Usage")
            .inner_size(300.0, 320.0)
            .resizable(false)
            .decorations(false)
            .always_on_top(true)
            .visible(false)
            .build()
            .expect("Failed to create popup window");

        let _ = window.move_window(Position::TrayCenter);
        let _ = window.show();
        let _ = window.set_focus();

        // Auto-dismiss on focus loss
        let win = window.clone();
        window.on_window_event(move |event| {
            if let WindowEvent::Focused(false) = event {
                let _ = win.hide();
            }
        });
    }
}

async fn poll_usage(app: &tauri::AppHandle) {
    let state = app.state::<Mutex<AppState>>();
    let client = {
        state.lock().unwrap().http_client.clone()
    };

    if let Ok(data) = fetch_fresh_usage(&client).await {
        {
            let mut s = state.lock().unwrap();
            s.last_usage = Some(data.clone());
        }
        let _ = app.emit("usage-updated", data);
    }
}
