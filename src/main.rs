#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use gettextrs::*;
use gtk::{gio, glib};

mod application;
mod config;
mod database;
mod models;
mod schema;
mod settings;
mod views;
mod widgets;

use application::Application;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE};

fn main() -> glib::ExitCode {
    pretty_env_logger::init();

    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).unwrap();
    textdomain(GETTEXT_PACKAGE).unwrap();
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8").unwrap();

    glib::set_application_name(&gettext("Read It Later"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    Application::run()
}
