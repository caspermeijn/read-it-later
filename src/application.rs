use self::config::{APP_ID, VERSION};
use crate::config;
use crate::database;
use crate::models::CACHE_DIR;
use crate::models::{Article, ArticleAction, ClientManager, SecretManager};
use crate::settings::{Key, SettingsManager};
use crate::widgets::{SettingsWidget, View, Window};
use anyhow::Result;
use async_std::sync::{Arc, Mutex};
use chrono::{TimeZone, Utc};
use futures::executor::ThreadPool;
use gettextrs::gettext;
use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::{Receiver, Sender};
use gtk::prelude::*;
use gtk_macros::{action, send, spawn};
use log::{error, info};
use std::{cell::RefCell, rc::Rc};
use url::Url;

use wallabag_api::types::Config;

pub enum Action {
    Articles(Box<ArticleAction>),
    SaveArticle(Url), // Save the pasted url
    SetView(View),
    PreviousView,
    Notify(String), // Notification message?
    SetClientConfig(Config),
    LoadArticles(Vec<Article>), // Post sync action
    Sync,
    Login,
    Logout,
}

pub struct Application {
    app: gtk::Application,
    window: Window,
    sender: Sender<Action>,
    receiver: RefCell<Option<Receiver<Action>>>,
    client: Arc<Mutex<ClientManager>>,
}

impl Application {
    pub fn new() -> Rc<Self> {
        gtk::Window::set_default_icon_name(APP_ID);

        let app = gtk::Application::new(Some(config::APP_ID), gio::ApplicationFlags::FLAGS_NONE);
        app.set_resource_base_path(Some("/com/belmoussaoui/ReadItLater"));

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

        std::fs::create_dir_all(&CACHE_DIR.to_str().unwrap()).unwrap();

        let receiver = self.receiver.borrow_mut().take().unwrap();
        receiver.attach(None, move |action| app.do_action(action));

        self.app.run();
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
            Action::LoadArticles(articles) => self.load_articles(articles),
            /* UI */
            Action::SetView(view) => self.window.set_view(view),
            Action::PreviousView => self.window.previous_view(),
            Action::Notify(err_msg) => self.window.notify(err_msg),
            /* Auth */
            Action::SetClientConfig(config) => self.set_client_config(config),
            Action::Sync => self.sync(),
            Action::Login => self.login(),
            Action::Logout => {
                if self.logout().is_err() {
                    send!(self.sender, Action::Notify("Failed to logout".to_string()));
                }
            }
        };
        glib::Continue(true)
    }

    fn do_article_action(&self, action: Box<ArticleAction>) {
        match *action {
            ArticleAction::Add(article) => self.add_article(article),
            ArticleAction::Open(article) => self.window.load_article(article),
            ArticleAction::Delete(article) => self.delete_article(article),
            ArticleAction::Archive(article) => self.archive_article(article),
            ArticleAction::Favorite(article) => self.favorite_article(article),
            ArticleAction::Update(article) => self.update_article(article),
        };
    }

    fn setup_gactions(&self) {
        // Quit
        action!(
            self.app,
            "quit",
            clone!(@strong self.app as app => move |_, _| {
                app.quit();
            })
        );
        // Settings
        action!(
            self.app,
            "settings",
            clone!(@strong self.window.widget as window, @strong self.client as client =>  move |_, _| {
                let settings_widget = SettingsWidget::new(client.clone());
                settings_widget.widget.set_transient_for(Some(&window));
                let size = window.size();
                settings_widget.widget.resize(size.0, size.1);
                settings_widget.widget.show();
            })
        );
        // About
        action!(
            self.app,
            "about",
            clone!(@strong self.window.widget as window => move |_, _| {
                Application::show_about_dialog(&window);
            })
        );
        action!(
            self.app,
            "new-article",
            clone!(@strong self.sender as sender => move |_, _| {
                send!(sender, Action::SetView(View::NewArticle));
            })
        );
        action!(
            self.app,
            "logout",
            clone!(@strong self.sender as sender => move |_, _| {
                send!(sender, Action::Logout);
            })
        );
        action!(
            self.app,
            "sync",
            clone!(@strong self.sender as sender => move |_, _| {
                send!(sender, Action::Sync);
            })
        );

        self.app.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
        self.app.set_accels_for_action("window.previous", &["Escape"]);

        self.app.set_accels_for_action("app.quit", &["<primary>q"]);
        self.app.set_accels_for_action("app.settings", &["<primary>comma"]);
        self.app.set_accels_for_action("app.new-article", &["<primary>N"]);
        self.app.set_accels_for_action("app.sync", &["F5"]);
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

        let style_manager = libhandy::StyleManager::default().unwrap();
        style_manager.set_color_scheme(libhandy::ColorScheme::PreferLight);
    }

    fn setup_css(&self) {
        let p = gtk::CssProvider::new();
        gtk::CssProvider::load_from_resource(&p, "/com/belmoussaoui/ReadItLater/style.css");
        if let Some(screen) = gtk::gdk::Screen::default() {
            gtk::StyleContext::add_provider_for_screen(&screen, &p, 500);
        }
    }

    /**
     * Auth
     */
    fn init_client(&self) {
        let username = SettingsManager::string(Key::Username);
        match SecretManager::is_logged(&username) {
            Ok(config) => {
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
        let logged_username = SettingsManager::string(Key::Username);

        spawn!(async move {
            let mut client = client.lock().await;
            match client.set_config(config.clone()).await {
                Ok(_) => {
                    if client.get_user().is_some() {
                        if config.username != logged_username {
                            SettingsManager::set_string(Key::Username, config.username.clone());
                        }
                        if let Err(err) = SecretManager::store_from_config(config) {
                            error!("Failed to store credentials {}", err);
                        }
                        send!(sender, Action::Login);
                    } else {
                        send!(sender, Action::Notify("Failed to log in".into()));
                        send!(sender, Action::SetView(View::Syncing(false)));
                    }
                }
                Err(err) => {
                    send!(sender, Action::Notify("Failed to log in".to_string()));
                    send!(sender, Action::SetView(View::Syncing(false)));
                    error!("Failed to setup a new client from current config: {}", err);
                }
            }
        });
    }

    fn login(&self) {
        self.sync();
        send!(self.sender, Action::SetView(View::Articles));
    }

    fn logout(&self) -> Result<()> {
        let username = SettingsManager::string(Key::Username);
        database::wipe()?;
        self.window.articles_view.clear();
        if SecretManager::logout(&username).is_ok() {
            SettingsManager::set_string(Key::Username, "".into());
            SettingsManager::set_integer(Key::LatestSync, 0);
        }
        send!(self.sender, Action::SetView(View::Login));
        Ok(())
    }

    fn sync(&self) {
        send!(self.sender, Action::SetView(View::Syncing(true)));
        let mut since = Utc.timestamp(0, 0);
        let last_sync = SettingsManager::integer(Key::LatestSync);
        if last_sync != 0 {
            since = Utc.timestamp(last_sync.into(), 0);
        }
        info!("Last sync was at {}", since);

        let client = self.client.clone();
        let sender = self.sender.clone();
        let now = Utc::now().timestamp();
        spawn!(async move {
            let client = client.lock().await;
            info!("Starting a new sync");
            match client.sync(since).await {
                Ok(articles) => {
                    send!(sender, Action::LoadArticles(articles));
                    SettingsManager::set_integer(Key::LatestSync, now as i32);
                }
                Err(err) => error!("Failed to sync {:#?}", err),
            };
            send!(sender, Action::SetView(View::Syncing(false)));
        });
    }

    /**
     *   Articles
     */

    fn add_article(&self, article: Article) {
        self.window.articles_view.add(&article);
    }

    fn load_articles(&self, articles: Vec<Article>) {
        let pool = ThreadPool::new().expect("Failed to build pool");
        let sender = self.sender.clone();
        spawn!(async move {
            let futures = async move {
                articles.iter().for_each(|article| {
                    match article.insert() {
                        Ok(_) => send!(sender, Action::Articles(Box::new(ArticleAction::Add(article.clone())))),
                        Err(_) => send!(sender, Action::Articles(Box::new(ArticleAction::Update(article.clone())))),
                    };
                })
            };
            pool.spawn_ok(futures);
        });
    }

    fn update_article(&self, article: Article) {
        self.window.articles_view.update(&article);
    }

    fn save_article(&self, url: Url) {
        info!("Saving new article \"{:#?}\"", url);
        send!(self.sender, Action::PreviousView);
        let client = self.client.clone();
        let sender = self.sender.clone();
        send!(sender, Action::SetView(View::Syncing(true)));
        spawn!(async move {
            let client = client.lock().await;
            client.save_url(url).await;
            send!(sender, Action::SetView(View::Syncing(false)));
            send!(sender, Action::Sync);
        });
    }

    fn archive_article(&self, article: Article) {
        info!("(Un)archiving the article \"{:#?}\" with ID: {}", article.title, article.id);
        send!(self.sender, Action::SetView(View::Syncing(true)));
        self.window.articles_view.archive(&article);

        let sender = self.sender.clone();
        let client = self.client.clone();
        spawn!(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            send!(sender, Action::SetView(View::Syncing(false)));
        });
    }

    fn favorite_article(&self, article: Article) {
        info!("(Un)favoriting the article \"{:#?}\" with ID: {}", article.title, article.id);
        send!(self.sender, Action::SetView(View::Syncing(true)));
        self.window.articles_view.favorite(&article);

        let sender = self.sender.clone();
        let client = self.client.clone();
        spawn!(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            send!(sender, Action::SetView(View::Syncing(false)));
        });
    }

    fn delete_article(&self, article: Article) {
        info!("Deleting the article \"{:#?}\" with ID: {}", article.title, article.id);
        send!(self.sender, Action::SetView(View::Syncing(true)));
        self.window.articles_view.delete(&article);

        let sender = self.sender.clone();
        let client = self.client.clone();
        let article_id: i32 = article.id;
        spawn!(async move {
            let client = client.lock().await;
            client.delete_entry(article_id).await;
            send!(sender, Action::SetView(View::Syncing(false)));
            send!(sender, Action::PreviousView);
        });
    }

    fn show_about_dialog(parent: &impl IsA<gtk::Window>) {
        let dialog = gtk::AboutDialog::builder()
            .logo_icon_name(APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/read-it-later/")
            .version(VERSION)
            .transient_for(parent)
            .translator_credits(&gettext("translator-credits"))
            .modal(true)
            .authors(vec!["Bilal Elmoussaoui".into(), "Casper Meijn".into()])
            .artists(vec!["Tobias Bernard".into()])
            .build();

        dialog.present();
    }
}
