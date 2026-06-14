import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type ThemePreference = "system" | "light" | "dark";

export interface WindowSettings {}

export interface EditorSettings {}

export interface AppSettings {
  theme: ThemePreference;
  window: WindowSettings;
  editor: EditorSettings;
}

type AppEvent =
  | { type: "themeChanged"; payload: { theme: ThemePreference } }
  | { type: "localeChanged"; payload: { locale: string } };

const APP_EVENT_NAME = "app-event";

let cachedSettings: AppSettings | null = null;

function defaultSettings(): AppSettings {
  return {
    theme: "system",
    window: {},
    editor: {},
  };
}

function applyTheme(theme: ThemePreference): void {
  document.documentElement.dataset.theme = theme;
}

export function getCachedSettings(): AppSettings {
  return cachedSettings ?? defaultSettings();
}

export async function getSettings(): Promise<AppSettings> {
  const settings = await invoke<AppSettings>("load_settings");
  cachedSettings = settings;
  return settings;
}

export async function setTheme(theme: ThemePreference): Promise<AppSettings> {
  const updated = await invoke<AppSettings>("update_theme", { theme });
  cachedSettings = updated;
  return updated;
}

export async function getSettingsFilePath(): Promise<string> {
  return invoke<string>("settings_file_path");
}

export async function initializeSettings(): Promise<AppSettings> {
  const settings = await getSettings();
  applyTheme(settings.theme);
  return settings;
}

export async function watchSettingsChanges(): Promise<() => void> {
  return listen<AppEvent>(APP_EVENT_NAME, (event) => {
    if (event.payload.type === "themeChanged") {
      const settings = getCachedSettings();
      cachedSettings = {
        ...settings,
        theme: event.payload.payload.theme,
      };
      applyTheme(event.payload.payload.theme);
    }
  });
}
