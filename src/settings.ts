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

export interface SaveSettingsInput {
  window: WindowSettings;
  editor: EditorSettings;
}

const SETTINGS_CHANGED_EVENT = "settings-changed";

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

export async function saveSettings(settings: SaveSettingsInput): Promise<AppSettings> {
  const saved = await invoke<AppSettings>("save_settings", { settings });
  cachedSettings = saved;
  return saved;
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
  return listen<AppSettings>(SETTINGS_CHANGED_EVENT, (event) => {
    cachedSettings = event.payload;
    applyTheme(event.payload.theme);
  });
}
