#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use gettextrs::*;
use gtk::gio;
use gtk::glib;

mod application;
mod config;
mod database;
mod models;
mod schema;
mod settings;
mod views;
mod widgets;

use application::Application;

use self::config::{GETTEXT_PACKAGE, LOCALEDIR, NAME_SUFFIX, RESOURCES_FILE};

fn main() {
    pretty_env_logger::init();

    gtk::init().expect("Unable to start GTK3");
    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).unwrap();
    textdomain(GETTEXT_PACKAGE).unwrap();

    glib::set_prgname(Some("read-it-later"));
    glib::set_application_name(&format!("Read It Later{}", NAME_SUFFIX));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    adw::init();
    webkit2gtk::WebView::new();

    let app = Application::new();
    app.run(app.clone());
}
