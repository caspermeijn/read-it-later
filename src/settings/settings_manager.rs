/*
Source: https://gitlab.gnome.org/World/Shortwave/blob/master/src/settings/settings_manager.rs
Thanks Felix!
*/

use gtk::gio;
use gtk::gio::prelude::*;
use log::error;

use crate::config;
use crate::settings::Key;

pub struct SettingsManager {}

impl SettingsManager {
    pub fn get_settings() -> gio::Settings {
        let app_id = config::APP_ID.trim_end_matches(".Devel");
        gio::Settings::new(app_id)
    }

    pub fn string(key: Key) -> String {
        let settings = Self::get_settings();
        settings.string(&key.to_string()).to_string()
    }

    pub fn set_string(key: Key, value: String) {
        let settings = Self::get_settings();
        if let Err(err) = settings.set_string(&key.to_string(), &value) {
            error!("Failed to save {} setting due to {}", key.to_string(), err);
        }
    }

    pub fn integer(key: Key) -> i32 {
        let settings = Self::get_settings();
        settings.int(&key.to_string())
    }

    pub fn set_integer(key: Key, value: i32) {
        let settings = Self::get_settings();
        if let Err(err) = settings.set_int(&key.to_string(), value) {
            error!("Failed to save {} setting due to {}", key.to_string(), err);
        }
    }
}
