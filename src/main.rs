#[macro_use]
extern crate log;

use gettextrs::*;
use libhandy::Column;

mod application;
mod config;
mod static_resources;
mod window;
mod window_state;

use application::Application;

fn main() {
    gtk::init().expect("Unable to start GTK3");
    // Prepare i18n
    setlocale(LocaleCategory::LcAll, "");
    bindtextdomain(config::GETTEXT_PACKAGE, config::LOCALEDIR);
    textdomain(config::GETTEXT_PACKAGE);

    static_resources::init().expect("Failed to initialize the resource file.");

    glib::set_application_name(&format!("Reading List{}", config::NAME_SUFFIX));
    glib::set_prgname(Some("reading-list"));
    Column::new();
    let app = Application::new();
    app.run();
}
