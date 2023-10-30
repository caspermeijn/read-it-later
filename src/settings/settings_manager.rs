// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// Source: https://gitlab.gnome.org/World/Shortwave/blob/master/src/settings/settings_manager.rs
// Thanks Felix!

use gio::prelude::*;
use gtk::gio;
use log::error;

use crate::{config, settings::Key};

pub struct SettingsManager {}

impl SettingsManager {
    pub fn get_settings() -> gio::Settings {
        gio::Settings::new(config::APP_ID)
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
