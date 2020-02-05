use crate::settings::{Key, SettingsManager};
use gtk::prelude::GtkWindowExt;

pub fn load(window: &gtk::ApplicationWindow) {
    let width = SettingsManager::get_integer(Key::WindowWidth);
    let height = SettingsManager::get_integer(Key::WindowHeight);

    if width > -1 && height > -1 {
        window.resize(width, height);
    }

    let x = SettingsManager::get_integer(Key::WindowX);
    let y = SettingsManager::get_integer(Key::WindowY);
    let is_maximized = SettingsManager::get_boolean(Key::IsMaximized);

    if x > -1 && y > -1 {
        window.move_(x, y);
    } else if is_maximized {
        window.maximize();
    }
}

pub fn save(window: &gtk::ApplicationWindow) {
    let size = window.get_size();
    let position = window.get_position();

    SettingsManager::set_integer(Key::WindowWidth, size.0);
    SettingsManager::set_integer(Key::WindowHeight, size.1);

    SettingsManager::set_boolean(Key::IsMaximized, window.is_maximized());

    SettingsManager::set_integer(Key::WindowX, position.0);
    SettingsManager::set_integer(Key::WindowY, position.1);
}
