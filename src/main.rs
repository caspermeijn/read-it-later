#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate glib;
#[macro_use]
extern crate gtk_macros;

use gettextrs::*;

mod application;
mod config;
mod database;
mod models;
mod schema;
mod settings;
mod static_resources;
mod views;
mod widgets;

use application::Application;

fn main() {
    pretty_env_logger::init();

    gtk::init().expect("Unable to start GTK3");
    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR).unwrap();
    textdomain(config::GETTEXT_PACKAGE).unwrap();

    glib::set_prgname(Some("read-it-later"));
    glib::set_application_name(&format!("Read It Later{}", config::NAME_SUFFIX));

    static_resources::init().expect("Failed to initialize the resource file.");
    libhandy::init();
    webkit2gtk::WebView::new();

    let app = Application::new();
    app.run(app.clone());
}
