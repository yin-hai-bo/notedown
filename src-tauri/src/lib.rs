mod menu;
mod settings;

use menu::AppMenu;
use settings::{AppSettings, SaveSettingsInput, SettingsError, ThemePreference};
use tauri::{Manager, State};

#[tauri::command]
fn load_settings(app: tauri::AppHandle) -> Result<AppSettings, SettingsError> {
    settings::load_settings(&app)
}

#[tauri::command]
fn save_settings(
    app: tauri::AppHandle,
    settings: SaveSettingsInput,
) -> Result<AppSettings, SettingsError> {
    let saved = settings::save_settings(&app, settings)?;
    Ok(saved)
}

#[tauri::command]
fn update_theme(
    app: tauri::AppHandle,
    app_menu: State<'_, AppMenu>,
    theme: ThemePreference,
) -> Result<AppSettings, SettingsError> {
    let updated = settings::update_theme(&app, theme)?;
    app_menu.sync_theme_menu_items(&app, &updated.theme);
    Ok(updated)
}

#[tauri::command]
fn settings_file_path(app: tauri::AppHandle) -> Result<String, SettingsError> {
    let path = settings::resolve_settings_file_path(&app)?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
            if let Ok(settings) = settings::load_settings(app.handle()) {
                app_menu.sync_theme_menu_items(app.handle(), &settings.theme);
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            update_theme,
            settings_file_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
