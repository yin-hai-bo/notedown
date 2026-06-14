use serde::{Deserialize, Serialize, Serializer};
use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager, Runtime};

use crate::app_event::{events_for_settings_change, AppEventBus};

const SETTINGS_FILE_NAME: &str = "settings.json";
const SETTINGS_VERSION: u32 = 1;
static SETTINGS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowSettings {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct EditorSettings {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: ThemePreference,
    pub window: WindowSettings,
    pub editor: EditorSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            window: WindowSettings::default(),
            editor: EditorSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SettingsFile {
    pub version: u32,
    pub settings: AppSettings,
}

impl Default for SettingsFile {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            settings: AppSettings::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsError {
    pub code: &'static str,
    pub message: String,
}

impl SettingsError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Serialize for SettingsError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct ErrorPayload<'a> {
            code: &'a str,
            message: &'a str,
        }

        ErrorPayload {
            code: self.code,
            message: &self.message,
        }
        .serialize(serializer)
    }
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for SettingsError {}

fn settings_file_path_for_dir(base_dir: &Path) -> PathBuf {
    base_dir.join(SETTINGS_FILE_NAME)
}

fn settings_lock() -> &'static Mutex<()> {
    SETTINGS_LOCK.get_or_init(|| Mutex::new(()))
}

pub fn resolve_settings_file_path<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<PathBuf, SettingsError> {
    let base_dir = app.path().app_local_data_dir().map_err(|error| {
        SettingsError::new(
            "path resolve failed",
            format!("failed to resolve app local data directory: {error}"),
        )
    })?;

    Ok(settings_file_path_for_dir(&base_dir))
}

fn ensure_parent_dir(path: &Path) -> Result<(), SettingsError> {
    match path.parent() {
        Some(parent) => fs::create_dir_all(parent).map_err(|error| {
            SettingsError::new(
                "create dir failed",
                format!(
                    "failed to create settings directory {}: {error}",
                    parent.display()
                ),
            )
        }),
        None => Err(SettingsError::new(
            "create dir failed",
            format!("settings path has no parent directory: {}", path.display()),
        )),
    }
}

fn parse_settings_file(contents: &str, path: &Path) -> Result<SettingsFile, SettingsError> {
    let parsed: SettingsFile = serde_json::from_str(contents).map_err(|error| {
        SettingsError::new(
            "parse failed",
            format!("failed to parse settings file {}: {error}", path.display()),
        )
    })?;

    if parsed.version != SETTINGS_VERSION {
        return Err(SettingsError::new(
            "parse failed",
            format!(
                "unsupported settings version {} in {}",
                parsed.version,
                path.display()
            ),
        ));
    }

    Ok(parsed)
}

fn read_settings_from_path(path: &Path) -> Result<AppSettings, SettingsError> {
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let contents = fs::read_to_string(path).map_err(|error| {
        SettingsError::new(
            "read failed",
            format!("failed to read settings file {}: {error}", path.display()),
        )
    })?;

    let parsed = parse_settings_file(&contents, path)?;
    Ok(parsed.settings)
}

fn temp_settings_path(path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    path.with_extension(format!("json.tmp.{}.{}", std::process::id(), timestamp))
}

fn write_settings_to_path(path: &Path, settings: &AppSettings) -> Result<(), SettingsError> {
    ensure_parent_dir(path)?;

    let temp_path = temp_settings_path(path);
    let file = SettingsFile {
        version: SETTINGS_VERSION,
        settings: settings.clone(),
    };
    let payload = serde_json::to_vec_pretty(&file).map_err(|error| {
        SettingsError::new(
            "write failed",
            format!(
                "failed to serialize settings for {}: {error}",
                path.display()
            ),
        )
    })?;

    fs::write(&temp_path, payload).map_err(|error| {
        SettingsError::new(
            "write failed",
            format!(
                "failed to write temporary settings file {}: {error}",
                temp_path.display()
            ),
        )
    })?;

    fs::rename(&temp_path, path).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        SettingsError::new(
            "rename failed",
            format!(
                "failed to replace settings file {} with {}: {error}",
                path.display(),
                temp_path.display()
            ),
        )
    })?;

    Ok(())
}

pub fn load_settings<R: Runtime>(app: &AppHandle<R>) -> Result<AppSettings, SettingsError> {
    let _guard = settings_lock().lock().expect("settings mutex poisoned");
    let path = resolve_settings_file_path(app)?;
    read_settings_from_path(&path)
}

pub fn update_theme(
    app: &AppHandle<impl Runtime>,
    theme: ThemePreference,
) -> Result<AppSettings, SettingsError> {
    let _guard = settings_lock().lock().expect("settings mutex poisoned");
    let path = resolve_settings_file_path(app)?;
    let previous = read_settings_from_path(&path)?;
    let mut settings = previous.clone();
    settings.theme = theme;
    write_settings_to_path(&path, &settings)?;
    let events = events_for_settings_change(&previous, &settings);
    let event_bus = app.state::<AppEventBus>();
    event_bus.publish_all(app, events)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    fn unique_test_dir() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("notedown-settings-tests-{id}"))
    }

    fn cleanup_dir(path: &Path) {
        if path.exists() {
            fs::remove_dir_all(path).expect("failed to remove test directory");
        }
    }

    #[test]
    fn missing_settings_file_returns_defaults() {
        let dir = unique_test_dir();
        let path = settings_file_path_for_dir(&dir);

        let settings = read_settings_from_path(&path).expect("expected default settings");

        assert_eq!(settings, AppSettings::default());
    }

    #[test]
    fn valid_json_round_trips() {
        let dir = unique_test_dir();
        let path = settings_file_path_for_dir(&dir);
        let settings = AppSettings {
            theme: ThemePreference::Dark,
            ..AppSettings::default()
        };

        write_settings_to_path(&path, &settings).expect("expected write to succeed");
        let loaded = read_settings_from_path(&path).expect("expected read to succeed");

        assert_eq!(loaded, settings);
        cleanup_dir(&dir);
    }

    #[test]
    fn invalid_json_returns_parse_error_without_overwriting() {
        let dir = unique_test_dir();
        let path = settings_file_path_for_dir(&dir);
        ensure_parent_dir(&path).expect("expected parent dir creation");
        fs::write(&path, b"{ this is not valid json")
            .expect("expected invalid test file to be written");

        let error = read_settings_from_path(&path).expect_err("expected parse failure");
        let contents = fs::read_to_string(&path).expect("expected original file to remain");

        assert_eq!(error.code, "parse failed");
        assert_eq!(contents, "{ this is not valid json");
        cleanup_dir(&dir);
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = unique_test_dir();
        let path = settings_file_path_for_dir(&dir);

        write_settings_to_path(&path, &AppSettings::default()).expect("expected write to succeed");

        assert!(path.exists(), "settings file should exist after save");
        cleanup_dir(&dir);
    }

    #[test]
    fn settings_file_contains_version_field() {
        let dir = unique_test_dir();
        let path = settings_file_path_for_dir(&dir);

        write_settings_to_path(&path, &AppSettings::default()).expect("expected write to succeed");
        let contents = fs::read_to_string(&path).expect("expected settings file contents");
        let parsed: serde_json::Value =
            serde_json::from_str(&contents).expect("expected valid json in settings file");

        assert_eq!(parsed["version"], SETTINGS_VERSION);
        assert!(
            parsed.get("settings").is_some(),
            "settings object should be present"
        );
        cleanup_dir(&dir);
    }
}
