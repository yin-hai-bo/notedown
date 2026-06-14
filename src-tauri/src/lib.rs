mod app_event;
mod menu;
mod settings;

use app_event::{AppEvent, AppEventBus};
use menu::AppMenu;
use settings::{AppSettings, SettingsError, ThemePreference};
use tauri::Manager;

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, SettingsError> {
    settings::load_settings(&app)
}

#[tauri::command]
fn update_theme(
    app: tauri::AppHandle,
    theme: ThemePreference,
) -> Result<AppSettings, SettingsError> {
    settings::update_theme(&app, theme)
}

#[tauri::command]
fn settings_file_path(app: tauri::AppHandle) -> Result<String, SettingsError> {
    let path = settings::resolve_settings_file_path(&app)?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppEventBus)
        .manage(AppMenu)
        .menu(|app| {
            let app_menu = app.state::<AppMenu>();
            app_menu.build(app)
        })
        .on_menu_event(|app, event| {
            let app_menu = app.state::<AppMenu>();
            app_menu.handle_event(app, event);
        })
        .setup(|app| {
            let app_menu = app.state::<AppMenu>();
            app_menu.register_app_event_listener(app.handle());

            if let Ok(settings) = settings::load_settings(app.handle()) {
                let event_bus = app.state::<AppEventBus>();
                event_bus.publish(
                    app.handle(),
                    AppEvent::ThemeChanged {
                        theme: settings.theme,
                    },
                )?;
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            update_theme,
            settings_file_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
