use gio::prelude::*;
use gtk::prelude::*;
use std::env;

use glib::{Receiver, Sender};
use std::{cell::RefCell, rc::Rc};

use crate::config;
use crate::models::{Article, ClientManager};
use crate::widgets::{View, Window};

use wallabag_api::types::Config;

pub enum Action {
    SetClientConfig(Config),
    LoadArticle(Article),
    PreviousView,
}

pub struct Application {
    app: gtk::Application,
    window: Window,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    client: RefCell<ClientManager>,
}

impl Application {
    pub fn new() -> Rc<Self> {
        let app = gtk::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE).unwrap();

        let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let receiver = RefCell::new(Some(r));

        let window = Window::new(sender.clone());

        let application = Rc::new(Self {
            app,
            window,
            sender,
            receiver,
            client: RefCell::new(ClientManager::new()),
        });

        application.setup_gactions();
        application.setup_signals();
        application.setup_css();
        application.setup_client();
        application
    }

    pub fn run(&self, app: Rc<Self>) {
        info!("Read It Later{} ({})", config::NAME_SUFFIX, config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        let app = app.clone();
        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.do_action(action));

        let args: Vec<String> = env::args().collect();
        self.app.run(&args);
    }

    fn do_action(&self, action: Action) -> glib::Continue {
        match action {
            Action::SetClientConfig(config) => {
                self.client.borrow_mut().set_config(config);
                self.window.set_view(View::Syncing);
            }
            Action::LoadArticle(article) => {
                self.window.load_article(article);
            }
            Action::PreviousView => {
                self.window.set_view(View::Unread);
            }
        };
        glib::Continue(true)
    }

    fn setup_gactions(&self) {
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

        let sender = self.sender.clone();
        self.add_gaction("back", move |_, _| {
            sender.send(Action::PreviousView).expect("Failed to trigger previous view action");
        });
        self.app.set_accels_for_action("app.back", &["escape"]);
    }

    fn setup_signals(&self) {
        let window = self.window.widget.clone();
        self.app.connect_activate(move |app| {
            window.set_application(Some(app));
            app.add_window(&window);
            window.present();
        });

        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/shortcuts.ui");
        let dialog: gtk::ShortcutsWindow = builder.get_object("shortcuts").unwrap();
        self.window.widget.set_help_overlay(Some(&dialog));
    }

    fn add_gaction<F>(&self, name: &str, action: F)
    where
        for<'r, 's> F: Fn(&'r gio::SimpleAction, Option<&'s glib::Variant>) + 'static,
    {
        let simple_action = gio::SimpleAction::new(name, None);
        simple_action.connect_activate(action);
        self.app.add_action(&simple_action);
    }

    fn setup_css(&self) {
        if let Some(theme) = gtk::IconTheme::get_default() {
            theme.add_resource_path("/com/belmoussaoui/ReadItLater/icons");
        }

        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/com/belmoussaoui/ReadItLater/style.css");
        if let Some(screen) = gdk::Screen::get_default() {
            gtk::StyleContext::add_provider_for_screen(&screen, &p, 500);
        }
    }

    fn setup_client(&self) {
        /*
        let client_config = Config {
            client_id: env::var("WALLABAG_CLIENT_ID").expect("WALLABAG_CLIENT_ID not set"),
            client_secret: env::var("WALLABAG_CLIENT_SECRET").expect("WALLABAG_CLIENT_SECRET not set"),
            username: env::var("WALLABAG_USERNAME").expect("WALLABAG_USERNAME not set"),
            password: env::var("WALLABAG_PASSWORD").expect("WALLABAG_PASSWORD not set"),
            base_url: env::var("WALLABAG_URL").expect("WALLABAG_URL not set"),
        };
        */
    }
}
