use chrono::{TimeZone, Utc};
use futures::lock::Mutex;
use gio::prelude::*;
use glib::futures::FutureExt;
use glib::{Receiver, Sender};
use gtk::prelude::*;
use std::env;
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};
use url::Url;

use crate::config;
use crate::models::{Article, ArticleAction, ClientManager, SecretManager};
use crate::settings::{Key, SettingsManager};
use crate::widgets::{SettingsWidget, View, Window};

use wallabag_api::types::Config;

pub enum Action {
    Articles(ArticleAction),
    SaveArticle(Url), // Save the pasted url
    SetView(View),
    PreviousView,
    Notify(String), // Notification message?
    SetClientConfig(Config),
    Login,
    Logout,
}

pub struct Application {
    app: gtk::Application,
    window: Rc<Window>,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    client: Arc<Mutex<ClientManager>>,
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
            client: Arc::new(Mutex::new(ClientManager::new(sender.clone()))),
            sender,
            receiver,
        });
        application.init();
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

    fn init(&self) {
        self.setup_gactions();
        self.setup_signals();
        self.setup_css();
        self.init_client();
    }

    fn do_action(&self, action: Action) -> glib::Continue {
        match action {
            /* Articles */
            Action::Articles(article_action) => self.do_article_action(article_action),
            Action::SaveArticle(url) => self.save_article(url),
            /* UI */
            Action::SetView(view) => self.window.set_view(view),
            Action::PreviousView => self.window.previous_view(),
            Action::Notify(err_msg) => self.window.notify(err_msg),
            /* Auth */
            Action::SetClientConfig(config) => self.set_client_config(config),
            Action::Login => self.login(),
            Action::Logout => self.logout(),
        };
        glib::Continue(true)
    }

    fn do_article_action(&self, action: ArticleAction) {
        match action {
            ArticleAction::Open(article) => self.window.load_article(article),
            ArticleAction::Delete(article) => self.delete_article(article),
            ArticleAction::Archive(article) => self.archive_article(article),
            ArticleAction::Favorite(article) => self.favorite_article(article),
            _ => (),
        };
    }

    fn setup_gactions(&self) {
        // Quit
        let app = self.app.clone();
        let sender = self.sender.clone();
        action!(self.app, "quit", move |_, _| app.quit());
        // Settings
        let weak_window = self.window.widget.downgrade();
        let client = self.client.clone();
        action!(self.app, "settings", move |_, _| {
            let settings_widget = SettingsWidget::new(client.clone());
            if let Some(window) = weak_window.upgrade() {
                settings_widget.widget.set_transient_for(Some(&window));
                let size = window.get_size();
                settings_widget.widget.resize(size.0, size.1);
            }
            settings_widget.widget.show();
        });
        // About
        let weak_window = self.window.widget.downgrade();
        action!(self.app, "about", move |_, _| {
            let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/about_dialog.ui");
            get_widget!(builder, gtk::AboutDialog, about_dialog);

            if let Some(window) = weak_window.upgrade() {
                about_dialog.set_transient_for(Some(&window));
            }

            about_dialog.connect_response(|dialog, _| dialog.destroy());
            about_dialog.show();
        });
        action!(
            self.app,
            "new-article",
            clone!(sender => move |_, _| {
                send!(sender, Action::SetView(View::NewArticle));
            })
        );
        action!(self.app, "logout", clone!(sender => move |_, _| send!(sender, Action::Logout)));
        action!(self.app, "back", clone!(sender => move |_, _| send!(sender, Action::PreviousView)));

        self.app.set_accels_for_action("app.back", &["Escape"]);
        self.app.set_accels_for_action("app.quit", &["<primary>q"]);
        self.app.set_accels_for_action("app.settings", &["<primary>comma"]);
        self.app.set_accels_for_action("app.new-article", &["<primary>N"]);
        // Articles
        self.app.set_accels_for_action("article.delete", &["Delete"]);
        self.app.set_accels_for_action("article.favorite", &["<primary><alt>F"]);
        self.app.set_accels_for_action("article.archive", &["<primary><alt>A"]);
        self.app.set_accels_for_action("article.open", &["<primary>O"]);
        self.app.set_accels_for_action("article.search", &["<primary>F"]);
    }

    fn setup_signals(&self) {
        let window = self.window.widget.clone();
        self.app.connect_activate(move |app| {
            window.set_application(Some(app));
            app.add_window(&window);
            window.present();
        });

        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/shortcuts.ui");
        get_widget!(builder, gtk::ShortcutsWindow, shortcuts);
        self.window.widget.set_help_overlay(Some(&shortcuts));
        self.app.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);

        if let Some(gtk_settings) = gtk::Settings::get_default() {
            SettingsManager::bind_property(Key::DarkMode, &gtk_settings, "gtk-application-prefer-dark-theme");
        }
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

    /**
     * Auth
     */
    fn init_client(&self) {
        let username = SettingsManager::get_string(Key::Username);
        match SecretManager::is_logged(&username) {
            Some(config) => {
                // Stored Config from Secret Service
                send!(self.sender, Action::SetView(View::Articles));
                self.set_client_config(config);
            }
            _ => send!(self.sender, Action::SetView(View::Login)),
        };
    }

    fn set_client_config(&self, config: Config) {
        send!(self.sender, Action::SetView(View::Syncing(true)));
        let client = self.client.clone();
        let sender = self.sender.clone();
        let logged_username = SettingsManager::get_string(Key::Username);

        spawn!(async move {
            client
                .lock()
                .then(async move |mut guard| {
                    if let Err(err) = guard.set_config(config.clone()).await {
                        send!(sender, Action::SetView(View::Syncing(false)));
                        error!("Failed to setup a new client from current config: {}", err);
                    }
                    if let Some(_) = guard.get_user() {
                        if config.username != logged_username {
                            SettingsManager::set_string(Key::Username, config.username.clone());
                            SecretManager::store_from_config(config);
                        }
                        send!(sender, Action::Login);
                    } else {
                        send!(sender, Action::Notify("Failed to log in".into()));
                        send!(sender, Action::SetView(View::Syncing(false)));
                    }
                })
                .await;
        });
    }

    fn login(&self) {
        self.sync();
        send!(self.sender, Action::SetView(View::Articles));
    }

    fn logout(&self) {
        SettingsManager::set_string(Key::Username, "".into());
        send!(self.sender, Action::SetView(View::Login));
    }

    fn sync(&self) {
        self.window.set_view(View::Syncing(true));
        let mut since = Utc.timestamp(0, 0);
        let last_sync = SettingsManager::get_integer(Key::LatestSync);
        if last_sync != 0 {
            since = Utc.timestamp(last_sync.into(), 0);
        }
        info!("Last sync was at {}", since);

        let client = self.client.clone();
        let sender = self.sender.clone();
        spawn!(async move {
            client
                .lock()
                .then(async move |guard| {
                    info!("Starting a new sync");
                    match guard.sync(since).await {
                        Ok(entries) => {
                            entries.into_iter().for_each(|entry| {
                                let sender_clone = sender.clone();
                                let article = Article::from(entry);
                                gtk::idle_add(move || {
                                    /*match article.insert() {
                                        // Ok(_) => send!(sender_clone, Action::AddArticle(article.clone())),
                                        // Err(_) => send!(sender_clone, Action::UpdateArticle(article.clone())),
                                    };
                                    */
                                    glib::Continue(false)
                                });
                            });

                            let now = Utc::now().timestamp();
                            SettingsManager::set_integer(Key::LatestSync, now as i32);
                        }
                        Err(err) => error!("Failed to sync {:#?}", err),
                    };
                    send!(sender, Action::SetView(View::Syncing(false)));
                })
                .await;
        });
    }

    /**
     *   Articles
     */

    fn save_article(&self, url: Url) {
        debug!("Saving new article \"{:#?}\"", url);
        let client = self.client.clone();
        let sender = self.sender.clone();
        send!(sender, Action::SetView(View::Syncing(true)));
        spawn!(async move {
            client
                .lock()
                .then(async move |guard| {
                    guard.save_url(url).await;
                    send!(sender, Action::SetView(View::Syncing(false)));
                    send!(sender, Action::PreviousView);
                })
                .await
        });
    }

    fn archive_article(&self, article: Article) {
        debug!("(Un)archiving the article \"{:#?}\" with ID: {}", article.title, article.id);
        send!(self.sender, Action::SetView(View::Syncing(true)));
        self.window.articles_view.archive(&article);

        let sender = self.sender.clone();
        let client = self.client.clone();
        spawn!(async move {
            client
                .lock()
                .then(async move |guard| {
                    guard.update_entry(article.id, article.get_patch()).await;
                    send!(sender, Action::SetView(View::Syncing(false)));
                })
                .await
        });
    }

    fn favorite_article(&self, article: Article) {
        debug!("(Un)favoriting the article \"{:#?}\" with ID: {}", article.title, article.id);
        send!(self.sender, Action::SetView(View::Syncing(true)));
        self.window.articles_view.favorite(&article);

        let sender = self.sender.clone();
        let client = self.client.clone();
        spawn!(async move {
            client
                .lock()
                .then(async move |guard| {
                    guard.update_entry(article.id, article.get_patch()).await;
                    send!(sender, Action::SetView(View::Syncing(false)));
                })
                .await
        });
    }

    fn delete_article(&self, article: Article) {
        debug!("Deleting the article \"{:#?}\" with ID: {}", article.title, article.id);
        send!(self.sender, Action::SetView(View::Syncing(true)));
        self.window.articles_view.delete(&article);

        let sender = self.sender.clone();
        let client = self.client.clone();
        let article_id: i32 = article.id;
        spawn!(async move {
            client
                .lock()
                .then(async move |guard| {
                    guard.delete_entry(article_id).await;
                    send!(sender, Action::SetView(View::Syncing(false)));
                    send!(sender, Action::PreviousView);
                })
                .await
        });
    }
}
