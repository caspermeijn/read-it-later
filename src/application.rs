use gio::prelude::*;
use gtk::prelude::*;
use std::env;

use crate::config;
use crate::widgets::Window;

pub struct Application {
    app: gtk::Application,
    window: Window,
}

impl Application {
    pub fn new() -> Self {
        let app = gtk::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE).unwrap();
        let window = Window::new();

        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/shortcuts.ui");
        let dialog: gtk::ShortcutsWindow = builder.get_object("shortcuts").unwrap();
        window.widget.set_help_overlay(Some(&dialog));

        let application = Self { app, window };

        application.setup_gactions();
        application.setup_signals();
        application.setup_css();
        application
    }

    pub fn setup_gactions(&self) {
        // Quit
        let app = self.app.clone();
        self.add_gaction("quit", move |_, _| app.quit());
        self.app.set_accels_for_action("app.quit", &["<primary>q"]);

        // About
        let weak_window = self.window.widget.downgrade();
        self.add_gaction("about", move |_, _| {
            let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/about_dialog.ui");
            let about_dialog: gtk::AboutDialog = builder.get_object("about_dialog").unwrap();
            if let Some(window) = weak_window.upgrade() {
                about_dialog.set_transient_for(Some(&window));
            }

            about_dialog.connect_response(|dialog, _| dialog.destroy());
            about_dialog.show();
        });
        self.app.set_accels_for_action("app.about", &["<primary>comma"]);
    }

    pub fn setup_signals(&self) {
        let window = self.window.widget.clone();
        self.app.connect_activate(move |app| {
            window.set_application(Some(app));
            app.add_window(&window);
            window.present();
        });
    }

    fn add_gaction<F>(&self, name: &str, action: F)
    where
        for<'r, 's> F: Fn(&'r gio::SimpleAction, Option<&'s glib::Variant>) + 'static,
    {
        let simple_action = gio::SimpleAction::new(name, None);
        simple_action.connect_activate(action);
        self.app.add_action(&simple_action);
    }

    pub fn setup_css(&self) {
        if let Some(theme) = gtk::IconTheme::get_default() {
            theme.add_resource_path("/com/belmoussaoui/ReadItLater/icons");
        }

        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/com/belmoussaoui/ReadItLater/style.css");
        if let Some(screen) = gdk::Screen::get_default() {
            gtk::StyleContext::add_provider_for_screen(&screen, &p, 500);
        }
    }

    pub fn run(&self) {
        info!("Read It Later{} ({})", config::NAME_SUFFIX, config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        let args: Vec<String> = env::args().collect();
        self.app.run(&args);
    }
}
