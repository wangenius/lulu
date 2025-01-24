// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_updater::UpdaterExt;

mod agents;
mod commands;
mod llm;
mod plugins;

static LAST_WINDOW_ACTION: AtomicI64 = AtomicI64::new(0);

#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<bool, String> {
    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(update) => Ok(update.is_some()),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update {
        let mut downloaded = 0;
        let progress = move |chunk_length: usize, content_length: Option<u64>| {
            downloaded += chunk_length;
            println!("已下载 {downloaded} / {content_length:?}");
        };
        let finished = || println!("下载完成");

        update
            .download_and_install(progress, finished)
            .await
            .map_err(|e| e.to_string())?;

        println!("更新已安装");
        app.restart();
    }

    Ok(())
}

fn main() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if shortcut == &Shortcut::new(Some(Modifiers::ALT), Code::Space)
                        && matches!(event.state(), ShortcutState::Pressed)
                    {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap() {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::models::add_model,
            commands::models::remove_model,
            commands::models::list_models,
            commands::models::set_current_model,
            commands::models::update_model,
            commands::models::get_model,
            commands::chat::chat,
            commands::bots::add_bot,
            commands::bots::remove_bot,
            commands::bots::list_bots,
            commands::bots::set_current_bot,
            commands::bots::get_current_bot,
            commands::bots::update_bot,
            commands::bots::get_bot,

            commands::history::create_chat_history,
            commands::history::update_chat_history,
            commands::history::list_histories,
            commands::history::delete_history,
            commands::window::open_window,
            commands::window::hide_window,
            commands::agents::add_agent,
            commands::agents::remove_agent,
            commands::agents::execute_agent,
            commands::agents::list_agents,
            commands::agents::get_agent,
            commands::agents::update_agent,
            commands::plugins::get_plugin,
            commands::plugins::add_plugin,
            commands::plugins::remove_plugin,
            commands::plugins::list_plugins,
            commands::plugins::update_plugin,
            commands::plugins::execute_plugin,
            check_update,
            install_update
        ])
        .setup(|app| {
            let _ = app.handle();
            let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
            app.global_shortcut().register(shortcut)?;

            let menu = Menu::with_items(
                app,
                &[
                    &MenuItem::with_id(app, "show", "显示", true, None::<&str>)?,
                    &MenuItem::with_id(app, "hide", "隐藏", true, None::<&str>)?,
                    &MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?,
                ],
            )?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    if let Some(window) = app.get_webview_window("main") {
                        match event.id().as_ref() {
                            "quit" => std::process::exit(0),
                            "show" => {
                                window.show().unwrap();
                                window.set_focus().unwrap();
                            }
                            "hide" => {
                                window.hide().unwrap();
                            }
                            _ => {}
                        }
                    }
                })
                .tooltip("echo智能助手")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        match event {
                            tauri::tray::TrayIconEvent::Click {
                                button: tauri::tray::MouseButton::Left,
                                ..
                            } => {
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as i64;
                                let last = LAST_WINDOW_ACTION.load(Ordering::SeqCst);

                                if now - last < 500 {
                                    return;
                                }

                                LAST_WINDOW_ACTION.store(now, Ordering::SeqCst);

                                let visible = window.is_visible().unwrap();
                                if visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            tauri::tray::TrayIconEvent::Click {
                                button: tauri::tray::MouseButton::Right,
                                ..
                            } => {
                                // 在 Tauri 2.0 中，菜单会自动显示，不需要手动调用
                            }
                            _ => {}
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        }
        tauri::RunEvent::Ready => {
            if let Some(window) = app_handle.get_webview_window("main") {
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        let _ = window_handle.hide();
                        api.prevent_close();
                    }
                });
            }
        }
        _ => {}
    });
}
