// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2020 Julian Hofer <julian.git@mailbox.org>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::OnceCell;

use adw::{prelude::*, subclass::prelude::*};
use anyhow::Result;
use async_std::sync::{Arc, Mutex};
use chrono::{TimeZone, Utc};
use futures::executor::ThreadPool;
use gettextrs::gettext;
use glib::{clone, Sender};
use gtk::{gio, glib};
use log::{error, info};
use url::Url;
use wallabag_api::types::Config;

use crate::{
    config, database,
    models::{Account, Article, ArticleAction, ClientManager, SecretManager, CACHE_DIR},
    settings::{Key, SettingsManager},
    widgets::{SettingsWidget, View, Window},
};

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
mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Application {
        pub window: OnceCell<Window>,
        pub sender: OnceCell<Sender<Action>>,
        pub client: OnceCell<Arc<Mutex<ClientManager>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for Application {}

    impl WidgetImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();

            let app = self.obj();

            let (sender, receiver) = glib::MainContext::channel(Default::default());
            self.sender.set(sender.clone()).unwrap();
            receiver.attach(
                None,
                clone!(@strong app => move |action| app.do_action(action)),
            );

            let client = Arc::new(Mutex::new(ClientManager::new(sender.clone())));
            self.client.set(client).unwrap();

            let window = Window::new(sender.clone());
            self.window.set(window.clone()).unwrap();

            gtk::Window::set_default_icon_name(config::APP_ID);

            app.setup_gactions();
            app.init_client();

            app.add_window(&window);
            window.present();
        }
    }

    impl GtkApplicationImpl for Application {}

    impl AdwApplicationImpl for Application {}
}

glib::wrapper! {
    pub struct Application(ObjectSubclass<imp::Application>)
        @extends gtk::Widget, gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl Application {
    pub fn run() -> glib::ExitCode {
        info!("Read It Later ({})", config::APP_ID);
        info!("Version: {} ({})", config::VERSION, config::PROFILE);
        info!("Datadir: {}", config::PKGDATADIR);

        std::fs::create_dir_all(&*CACHE_DIR).unwrap();

        let app = glib::Object::builder::<Application>()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/com/belmoussaoui/ReadItLater")
            .build();

        app.run()
    }

    fn do_action(&self, action: Action) -> glib::ControlFlow {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap();
        match action {
            // Articles
            Action::Articles(article_action) => self.do_article_action(article_action),
            Action::SaveArticle(url) => self.save_article(url),
            Action::LoadArticles(articles) => self.load_articles(articles),
            // UI
            Action::SetView(view) => window.set_view(view),
            Action::PreviousView => window.previous_view(),
            Action::Notify(err_msg) => window.add_toast(adw::Toast::new(&err_msg)),
            // Auth
            Action::SetClientConfig(config) => self.set_client_config(config),
            Action::Sync => self.sync(),
            Action::Login => self.login(),
            Action::Logout => {
                if self.logout().is_err() {
                    sender
                        .send(Action::Notify(gettext("Failed to logout")))
                        .unwrap();
                }
            }
        };
        glib::ControlFlow::Continue
    }

    fn do_article_action(&self, action: Box<ArticleAction>) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        match *action {
            ArticleAction::Add(article) => self.add_article(article),
            ArticleAction::Open(article) => window.load_article(article),
            ArticleAction::Delete(article) => self.delete_article(article),
            ArticleAction::Archive(article) => self.archive_article(article),
            ArticleAction::Favorite(article) => self.favorite_article(article),
            ArticleAction::Update(article) => self.update_article(article),
        };
    }

    fn setup_gactions(&self) {
        self.add_action_entries([
            // Quit
            gio::ActionEntry::builder("quit")
                .activate(|app: &Application, _, _| {
                    app.quit();
                })
                .build(),
            // Settings
            gio::ActionEntry::builder("settings")
                .activate(|app: &Application, _, _| {
                    let imp = app.imp();
                    let window = imp.window.get().unwrap();
                    let client = imp.client.get().unwrap();
                    let settings_widget = SettingsWidget::new(client.clone());
                    settings_widget.set_transient_for(Some(window));
                    settings_widget.present();
                })
                .build(),
            // About
            gio::ActionEntry::builder("about")
                .activate(|app: &Application, _, _| {
                    let imp = app.imp();
                    let window = imp.window.get().unwrap();
                    Application::show_about_dialog(window);
                })
                .build(),
            // Log out
            gio::ActionEntry::builder("logout")
                .activate(|app: &Application, _, _| {
                    let imp = app.imp();
                    let sender = imp.sender.get().unwrap();
                    sender.send(Action::Logout).unwrap();
                })
                .build(),
            // Sync
            gio::ActionEntry::builder("sync")
                .activate(|app: &Application, _, _| {
                    let imp = app.imp();
                    let sender = imp.sender.get().unwrap();
                    sender.send(Action::Sync).unwrap();
                })
                .build(),
            // Login
            gio::ActionEntry::builder("login")
                .parameter_type(Some(&crate::models::Account::static_variant_type()))
                .activate(|app: &Application, _, parameter| {
                    let imp = app.imp();
                    let sender = imp.sender.get().unwrap();
                    let account: Account = parameter.unwrap().get().unwrap();
                    sender
                        .send(Action::SetClientConfig(account.into()))
                        .unwrap();
                })
                .build(),
        ]);

        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
        self.set_accels_for_action("win.previous", &["Escape"]);

        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("app.settings", &["<primary>comma"]);
        self.set_accels_for_action("win.new-article", &["<primary>N"]);
        self.set_accels_for_action("app.sync", &["F5"]);
        // Articles
        self.set_accels_for_action("article.delete", &["Delete"]);
        self.set_accels_for_action("article.favorite", &["<primary><alt>F"]);
        self.set_accels_for_action("article.archive", &["<primary><alt>A"]);
        self.set_accels_for_action("article.open", &["<primary>O"]);
        self.set_accels_for_action("article.search", &["<primary>F"]);
    }

    /// Auth
    fn init_client(&self) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap();

        let username = SettingsManager::string(Key::Username);
        match SecretManager::is_logged(&username) {
            Ok(config) => {
                sender.send(Action::SetView(View::Articles)).unwrap();
                self.set_client_config(config);
            }
            _ => sender.send(Action::SetView(View::Login)).unwrap(),
        };
    }

    fn set_client_config(&self, config: Config) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        sender.send(Action::SetView(View::Syncing(true))).unwrap();
        let logged_username = SettingsManager::string(Key::Username);

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
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
                        sender.send(Action::Login).unwrap();
                    } else {
                        sender
                            .send(Action::Notify(gettext("Failed to log in")))
                            .unwrap();
                        sender.send(Action::SetView(View::Syncing(false))).unwrap();
                    }
                }
                Err(err) => {
                    sender
                        .send(Action::Notify(gettext("Failed to log in")))
                        .unwrap();
                    sender.send(Action::SetView(View::Syncing(false))).unwrap();
                    error!("Failed to setup a new client from current config: {}", err);
                }
            }
        });
    }

    fn login(&self) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap();

        self.sync();
        sender.send(Action::SetView(View::Articles)).unwrap();
    }

    fn logout(&self) -> Result<()> {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap();

        let username = SettingsManager::string(Key::Username);
        database::wipe()?;
        window.articles_view().clear();
        if SecretManager::logout(&username).is_ok() {
            SettingsManager::set_string(Key::Username, "".into());
            SettingsManager::set_integer(Key::LatestSync, 0);
        }
        sender.send(Action::SetView(View::Login)).unwrap();
        Ok(())
    }

    fn sync(&self) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        sender.send(Action::SetView(View::Syncing(true))).unwrap();
        let mut since = Utc.timestamp_opt(0, 0).unwrap();
        let last_sync = SettingsManager::integer(Key::LatestSync);
        if last_sync != 0 {
            since = Utc.timestamp_opt(last_sync.into(), 0).unwrap();
        }
        info!("Last sync was at {}", since);

        let now = Utc::now().timestamp();
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            info!("Starting a new sync");
            match client.sync(since).await {
                Ok(articles) => {
                    sender.send(Action::LoadArticles(articles)).unwrap();
                    SettingsManager::set_integer(Key::LatestSync, now as i32);
                }
                Err(err) => error!("Failed to sync {:#?}", err),
            };
            sender.send(Action::SetView(View::Syncing(false))).unwrap();
        });
    }

    ///   Articles

    fn add_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        window.articles_view().add(&article);
    }

    fn load_articles(&self, articles: Vec<Article>) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();

        let pool = ThreadPool::new().expect("Failed to build pool");
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let futures = async move {
                articles.iter().for_each(|article| {
                    match article.insert() {
                        Ok(_) => sender
                            .send(Action::Articles(Box::new(ArticleAction::Add(
                                article.clone(),
                            ))))
                            .unwrap(),
                        Err(_) => sender
                            .send(Action::Articles(Box::new(ArticleAction::Update(
                                article.clone(),
                            ))))
                            .unwrap(),
                    };
                })
            };
            pool.spawn_ok(futures);
        });
    }

    fn update_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        window.articles_view().update(&article);
    }

    fn save_article(&self, url: Url) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!("Saving new article \"{:#?}\"", url);
        sender.send(Action::PreviousView).unwrap();
        sender.send(Action::SetView(View::Syncing(true))).unwrap();
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.save_url(url).await;
            sender.send(Action::SetView(View::Syncing(false))).unwrap();
            sender.send(Action::Sync).unwrap();
        });
    }

    fn archive_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!(
            "(Un)archiving the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        sender.send(Action::SetView(View::Syncing(true))).unwrap();
        window.articles_view().archive(&article);

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            sender.send(Action::SetView(View::Syncing(false))).unwrap();
        });
    }

    fn favorite_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!(
            "(Un)favoriting the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        sender.send(Action::SetView(View::Syncing(true))).unwrap();
        window.articles_view().favorite(&article);

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            sender.send(Action::SetView(View::Syncing(false))).unwrap();
        });
    }

    fn delete_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!(
            "Deleting the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        sender.send(Action::SetView(View::Syncing(true))).unwrap();
        window.articles_view().delete(&article);

        let article_id: i32 = article.id;
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.delete_entry(article_id).await;
            sender.send(Action::SetView(View::Syncing(false))).unwrap();
            sender.send(Action::PreviousView).unwrap();
        });
    }

    fn authors() -> Vec<&'static str> {
        env!("CARGO_PKG_AUTHORS").split(':').collect()
    }

    fn show_about_dialog(parent: &impl IsA<gtk::Window>) {
        let dialog = adw::AboutWindow::builder()
            .application_name(glib::application_name().unwrap())
            .application_icon(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/read-it-later/")
            .version(config::VERSION)
            .translator_credits(gettext("translator-credits"))
            .developers(Self::authors())
            .artists(["Tobias Bernard"])
            .transient_for(parent)
            .build();

        dialog.present();
    }
}
