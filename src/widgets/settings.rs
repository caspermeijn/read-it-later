use crate::settings::{Key, SettingsManager};
use gtk::prelude::*;

use wallabag_api::types::User;

pub struct SettingsWidget {
    pub widget: libhandy::PreferencesWindow,
    builder: gtk::Builder,
}

impl SettingsWidget {
    pub fn new(user: Option<Result<User, wallabag_api::errors::ClientError>>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/settings.ui");
        get_widget!(builder, libhandy::PreferencesWindow, settings_window);

        let window = Self {
            builder,
            widget: settings_window,
        };

        window.init(user);
        window.setup_signals();
        window
    }

    fn init(&self, user: Option<Result<User, wallabag_api::errors::ClientError>>) {
        get_widget!(self.builder, gtk::Entry, username_entry);
        get_widget!(self.builder, gtk::Entry, email_entry);
        get_widget!(self.builder, gtk::Label, created_at_label);
        get_widget!(self.builder, gtk::Label, updated_at_label);

        match user {
            Some(Ok(user)) => {
                username_entry.set_text(&user.username);
                email_entry.set_text(&user.email);
                if let Some(created_at) = user.created_at {
                    created_at_label.set_text(&created_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
                if let Some(updated_at) = user.updated_at {
                    updated_at_label.set_text(&updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
            }
            _ => {}
        };
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Switch, dark_mode_button);
        SettingsManager::bind_property(Key::DarkMode, &dark_mode_button, "active");
    }
}
