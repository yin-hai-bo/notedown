use serde::Serialize;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, MenuItemKind, PredefinedMenuItem, Submenu},
    AppHandle, Emitter, Runtime,
};

use crate::settings::{self, ThemePreference};

pub const MENU_ACTION_EVENT: &str = "menu-action";

const MENU_NEW: &str = "file.new";
const MENU_OPEN: &str = "file.open";
const MENU_SAVE: &str = "file.save";
const MENU_SAVE_AS: &str = "file.save_as";
const MENU_PREFERENCES: &str = "file.preferences";
const MENU_QUIT: &str = "file.quit";
const MENU_THEME_SYSTEM: &str = "theme.system";
const MENU_THEME_LIGHT: &str = "theme.light";
const MENU_THEME_DARK: &str = "theme.dark";

#[derive(Clone, Default)]
pub struct AppMenu;

#[derive(Clone, Serialize)]
pub struct MenuActionPayload {
    pub id: String,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn menu_text<'a>(_plain_text: &'a str, mnemonic_text: &'a str) -> &'a str {
    mnemonic_text
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
fn menu_text<'a>(plain_text: &'a str, _mnemonic_text: &'a str) -> &'a str {
    plain_text
}

impl AppMenu {
    pub fn build<R: Runtime>(&self, app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
        let file_menu = Submenu::with_items(
            app,
            menu_text("文件", "文件(&F)"),
            true,
            &[
                &MenuItem::with_id(
                    app,
                    MENU_NEW,
                    menu_text("新建", "新建(&N)"),
                    true,
                    Some("CmdOrCtrl+N"),
                )?,
                &MenuItem::with_id(
                    app,
                    MENU_OPEN,
                    menu_text("打开...", "打开(&O)..."),
                    true,
                    Some("CmdOrCtrl+O"),
                )?,
                &MenuItem::with_id(
                    app,
                    MENU_SAVE,
                    menu_text("保存", "保存(&S)"),
                    true,
                    Some("CmdOrCtrl+S"),
                )?,
                &MenuItem::with_id(
                    app,
                    MENU_SAVE_AS,
                    menu_text("另存为...", "另存为(&A)..."),
                    true,
                    None::<&str>,
                )?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(
                    app,
                    MENU_PREFERENCES,
                    menu_text("偏好设置...", "偏好设置(&P)..."),
                    true,
                    Some("CmdOrCtrl+,"),
                )?,
                &PredefinedMenuItem::separator(app)?,
                &MenuItem::with_id(
                    app,
                    MENU_QUIT,
                    menu_text("退出", "退出(&X)"),
                    true,
                    None::<&str>,
                )?,
            ],
        )?;

        let theme_menu = Submenu::with_items(
            app,
            menu_text("主题", "主题(&T)"),
            true,
            &[
                &CheckMenuItem::with_id(
                    app,
                    MENU_THEME_SYSTEM,
                    menu_text("跟随系统", "跟随系统(&S)"),
                    true,
                    true,
                    None::<&str>,
                )?,
                &CheckMenuItem::with_id(
                    app,
                    MENU_THEME_LIGHT,
                    menu_text("浅色", "浅色(&L)"),
                    true,
                    false,
                    None::<&str>,
                )?,
                &CheckMenuItem::with_id(
                    app,
                    MENU_THEME_DARK,
                    menu_text("深色", "深色(&D)"),
                    true,
                    false,
                    None::<&str>,
                )?,
            ],
        )?;

        Menu::with_items(app, &[&file_menu, &theme_menu])
    }

    pub fn handle_event<R: Runtime>(&self, app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
        match event.id().as_ref() {
            MENU_QUIT => app.exit(0),
            MENU_THEME_SYSTEM => self.handle_theme_menu_event(app, ThemePreference::System),
            MENU_THEME_LIGHT => self.handle_theme_menu_event(app, ThemePreference::Light),
            MENU_THEME_DARK => self.handle_theme_menu_event(app, ThemePreference::Dark),
            MENU_NEW | MENU_OPEN | MENU_SAVE | MENU_SAVE_AS | MENU_PREFERENCES => {
                let payload = MenuActionPayload {
                    id: event.id().as_ref().to_string(),
                };
                let _ = app.emit(MENU_ACTION_EVENT, payload);
            }
            _ => {}
        }
    }

    pub fn sync_theme_menu_items<R: Runtime>(&self, app: &AppHandle<R>, theme: &ThemePreference) {
        let Some(menu) = app.menu() else {
            return;
        };

        self.sync_theme_menu_item(&menu, MENU_THEME_SYSTEM, matches!(theme, ThemePreference::System));
        self.sync_theme_menu_item(&menu, MENU_THEME_LIGHT, matches!(theme, ThemePreference::Light));
        self.sync_theme_menu_item(&menu, MENU_THEME_DARK, matches!(theme, ThemePreference::Dark));
    }

    fn handle_theme_menu_event<R: Runtime>(&self, app: &AppHandle<R>, theme: ThemePreference) {
        self.sync_theme_menu_items(app, &theme);

        if settings::update_theme(app, theme).is_err() {
            if let Ok(settings) = settings::load_settings(app) {
                self.sync_theme_menu_items(app, &settings.theme);
            }
        }
    }

    fn sync_theme_menu_item<R: Runtime>(&self, menu: &Menu<R>, id: &str, checked: bool) {
        let Some(item) = self.find_menu_item_in_menu(menu, id) else {
            return;
        };

        let Some(check_item) = item.as_check_menuitem() else {
            return;
        };

        let _ = check_item.set_checked(checked);
    }

    fn find_menu_item_in_menu<R: Runtime>(&self, menu: &Menu<R>, id: &str) -> Option<MenuItemKind<R>> {
        for item in menu.items().ok()? {
            if item.id() == &id {
                return Some(item);
            }

            if let Some(submenu) = item.as_submenu() {
                if let Some(found) = self.find_menu_item_in_submenu(submenu, id) {
                    return Some(found);
                }
            }
        }

        None
    }

    fn find_menu_item_in_submenu<R: Runtime>(
        &self,
        submenu: &Submenu<R>,
        id: &str,
    ) -> Option<MenuItemKind<R>> {
        for item in submenu.items().ok()? {
            if item.id() == &id {
                return Some(item);
            }

            if let Some(child_submenu) = item.as_submenu() {
                if let Some(found) = self.find_menu_item_in_submenu(child_submenu, id) {
                    return Some(found);
                }
            }
        }

        None
    }
}
