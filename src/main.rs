#![feature(async_closure)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
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
extern crate strum_macros;

extern crate pretty_env_logger;
extern crate webkit2gtk;

use gettextrs::*;
use libhandy::Column;

#[macro_use]
mod utils;

mod application;
mod config;
mod database;
mod models;
mod schema;
mod settings;
mod static_resources;
mod views;
mod widgets;
mod window_state;

use application::Application;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    gtk::init().expect("Unable to start GTK3");
    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR);
    textdomain(config::GETTEXT_PACKAGE);

    glib::set_prgname(Some("read-it-later"));
    glib::set_application_name(&format!("Read It Later{}", config::NAME_SUFFIX));

    static_resources::init().expect("Failed to initialize the resource file.");
    Column::new(); // Due to libhandy not having a main func :(

    let app = Application::new();
    app.run(app.clone());
}
