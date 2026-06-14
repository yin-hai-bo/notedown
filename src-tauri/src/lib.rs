mod app_event;
mod document;
mod menu;
mod settings;

use app_event::{AppEvent, AppEventBus};
use document::{open_document, save_document, save_document_as};
use menu::AppMenu;
use settings::{AppSettings, SettingsError, ThemePreference};
use tauri::{window::Color, Manager, WebviewUrl, WebviewWindowBuilder};

const BG_LIGHT: Color = Color(255, 255, 255, 255);
const BG_DARK: Color = Color(17, 17, 17, 255);

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

fn theme_to_background_color(theme: &ThemePreference) -> Color {
    match theme {
        ThemePreference::Light => BG_LIGHT,
        ThemePreference::Dark => BG_DARK,
        ThemePreference::System => match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => BG_DARK,
            _ => BG_LIGHT,
        },
    }
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

            let settings = settings::load_settings(app.handle()).ok();

            let theme = settings
                .as_ref()
                .map(|s| &s.theme)
                .unwrap_or(&ThemePreference::System);
            let theme_str = serde_json::to_string(theme).unwrap_or_default();
            let init_script = format!("window.__INITIAL_THEME__ = {};", theme_str);
            let _ = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
                .title("notedown")
                .inner_size(800.0, 600.0)
                .background_color(theme_to_background_color(theme))
                .initialization_script(&init_script)
                .build()?;

            if let Some(settings) = settings {
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
            settings_file_path,
            open_document,
            save_document,
            save_document_as
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
