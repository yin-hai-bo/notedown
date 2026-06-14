use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};

use crate::settings::{AppSettings, SettingsError, ThemePreference};

pub const APP_EVENT_NAME: &str = "app-event";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum AppEvent {
    ThemeChanged { theme: ThemePreference },
    LocaleChanged { locale: String },
}

#[derive(Clone, Default)]
pub struct AppEventBus;

impl AppEventBus {
    pub fn publish<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        event: AppEvent,
    ) -> Result<(), SettingsError> {
        self.emit_app_events(app, &event)?;
        Ok(())
    }

    pub fn publish_all<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        events: impl IntoIterator<Item = AppEvent>,
    ) -> Result<(), SettingsError> {
        for event in events {
            self.publish(app, event)?;
        }

        Ok(())
    }

    fn emit_app_events<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        event: &AppEvent,
    ) -> Result<(), SettingsError> {
        app.emit(APP_EVENT_NAME, event).map_err(|error| {
            SettingsError::new(
                "event publish failed",
                format!("failed to emit app event {APP_EVENT_NAME}: {error}"),
            )
        })?;

        Ok(())
    }
}

pub fn events_for_settings_change(previous: &AppSettings, current: &AppSettings) -> Vec<AppEvent> {
    let mut events = Vec::new();

    if previous.theme != current.theme {
        events.push(AppEvent::ThemeChanged {
            theme: current.theme.clone(),
        });
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings_with_theme(theme: ThemePreference) -> AppSettings {
        AppSettings {
            theme,
            ..AppSettings::default()
        }
    }

    #[test]
    fn theme_change_emits_settings_and_theme_events() {
        let previous = settings_with_theme(ThemePreference::System);
        let current = settings_with_theme(ThemePreference::Dark);

        let events = events_for_settings_change(&previous, &current);

        assert_eq!(
            events,
            vec![AppEvent::ThemeChanged {
                theme: current.theme.clone(),
            },]
        );
    }

    #[test]
    fn unchanged_theme_does_not_emit_theme_event() {
        let previous = settings_with_theme(ThemePreference::Dark);
        let current = settings_with_theme(ThemePreference::Dark);

        let events = events_for_settings_change(&previous, &current);

        assert!(events.is_empty());
    }
}
