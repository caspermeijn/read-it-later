extern crate pretty_env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
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

extern crate webkit2gtk;

use gettextrs::*;
use libhandy::Column;

mod application;
mod config;
mod database;
mod models;
mod schema;
mod static_resources;
mod views;
mod widgets;
mod window_state;

use application::Application;

fn main() {
    pretty_env_logger::init();

    gtk::init().expect("Unable to start GTK3");
    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR);
    textdomain(config::GETTEXT_PACKAGE);

    static_resources::init().expect("Failed to initialize the resource file.");

    glib::set_application_name(&format!("Read It Later{}", config::NAME_SUFFIX));
    glib::set_prgname(Some("read-it-later"));
    Column::new();
    let app = Application::new();
    app.run(app.clone());
}
