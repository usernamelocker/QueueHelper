use std::sync::Arc;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

fn create_tray_icon() -> Image<'static> {
    let bytes = include_bytes!("../icons/32x32.png");
    let img = image::load_from_memory(bytes).expect("failed to decode tray icon").into_rgba8();
    let (width, height) = img.dimensions();
    Image::new_owned(img.into_raw(), width, height)
}

mod commands;
mod app;
mod core;
mod lcu;
mod models;
mod persistence;
mod automation;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_runtime_snapshot,
            commands::get_settings,
            commands::update_settings,
            commands::get_profiles,
            commands::update_profiles,
            commands::get_rules,
            commands::update_rules,
            commands::get_monitor_entries,
            commands::set_automation_paused,
            commands::set_auto_accept_enabled,
            commands::set_auto_ban_enabled,
            commands::set_auto_pick_enabled,
            commands::set_auto_hover_enabled,
            commands::set_active_profile,
            commands::set_monitor_auto_scroll,
            commands::set_monitor_scroll_top,
            commands::get_monitor_scroll_top,
        ])
        .setup(|app| {
            use tauri::Manager;

            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&data_dir).expect("failed to create app data dir");

            let shutdown = tokio_util::sync::CancellationToken::new();
            let context = Arc::new(
                tauri::async_runtime::block_on(crate::app::AppContext::new(data_dir, shutdown.clone(), Some(app.handle().clone())))
                    .expect("Failed to initialize app context"),
            );

            // ── Background Tasks ──
            let ctx = context.clone();
            tauri::async_runtime::spawn(async move {
                tokio::join!(
                    crate::core::state::run_state_reducer(ctx.clone(), ctx.shutdown.clone()),
                    crate::lcu::lockfile::run_lockfile_monitor(
                        ctx.bus.clone(),
                        ctx.state.clone(),
                        ctx.shutdown.clone(),
                    ),
                    crate::lcu::manager::run_connection_manager(ctx.clone(), ctx.shutdown.clone()),
                    crate::automation::run_auto_accept(ctx.clone(), ctx.shutdown.clone()),
                    crate::automation::run_auto_ban(ctx.clone(), ctx.shutdown.clone()),
                    crate::automation::run_auto_pick(ctx.clone(), ctx.shutdown.clone()),
                    crate::automation::run_auto_hover(ctx.clone(), ctx.shutdown.clone()),
                    crate::automation::run_rules_engine(ctx.clone(), ctx.shutdown.clone()),
                );
            });

            app.manage(context);

            // ── System Tray ──
            let show_hide = MenuItemBuilder::with_id("toggle", "Queue Helper").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_hide)
                .separator()
                .item(&quit)
                .build()?;

            let icon = create_tray_icon();

            TrayIconBuilder::new()
                .icon(icon)
                .tooltip("Queue Helper")
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "toggle" => {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = show_hide.set_text("Show Queue Helper");
                                let _ = window.hide();
                            } else {
                                let _ = show_hide.set_text("Hide Queue Helper");
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // ── Minimize to tray on close ──
            if let Some(window) = app.get_webview_window("main") {
                let w = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = w.hide();
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
