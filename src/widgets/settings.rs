use crate::settings::{Key, SettingsManager};
use gtk::prelude::*;

pub struct SettingsWidget {
    pub widget: libhandy::PreferencesWindow,
    builder: gtk::Builder,
}

impl SettingsWidget {
    pub fn new() -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/settings.ui");
        get_widget!(builder, libhandy::PreferencesWindow, settings_window);

        let window = Self {
            builder,
            widget: settings_window,
        };

        window.setup_signals();
        window
    }

    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Switch, dark_mode_button);
        SettingsManager::bind_property(Key::DarkMode, &dark_mode_button, "active");
    }
}
