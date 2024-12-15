// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2020 Julian Hofer <julian.git@mailbox.org>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::OnceCell;

use adw::{prelude::*, subclass::prelude::*};
use anyhow::Result;
use async_std::{
    channel::Sender,
    sync::{Arc, Mutex},
};
use chrono::{TimeZone, Utc};
use futures::executor::ThreadPool;
use gettextrs::gettext;
use gtk::{gio, glib};
use log::{error, info};
use url::Url;
use wallabag_api::types::Config;

use crate::{
    config, database,
    models::{Account, Article, ArticleAction, ClientManager, SecretManager},
    settings::{Key, SettingsManager},
    widgets::{SettingsWidget, View, Window},
};

pub enum Action {
    Articles(Box<ArticleAction>),
    SaveArticle(Url), // Save the pasted url
    SetView(View),
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

            if let Some(window) = self.window.get() {
                window.present();
                return;
            }

            let (sender, receiver) = async_std::channel::unbounded();
            self.sender.set(sender.clone()).unwrap();

            let ctx = glib::MainContext::default();
            ctx.spawn_local(glib::clone!(
                #[strong]
                app,
                async move {
                    while let Ok(action) = receiver.recv().await {
                        app.do_action(action).await;
                    }
                }
            ));

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

        let app = glib::Object::builder::<Application>()
            .property("application-id", config::APP_ID)
            .property("resource-base-path", "/com/belmoussaoui/ReadItLater")
            .build();

        app.run()
    }

    async fn do_action(&self, action: Action) -> glib::ControlFlow {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap();
        match action {
            // Articles
            Action::Articles(article_action) => self.do_article_action(article_action).await,
            Action::SaveArticle(url) => self.save_article(url).await,
            Action::LoadArticles(articles) => self.load_articles(articles),
            // UI
            Action::SetView(view) => window.set_view(view),
            Action::Notify(err_msg) => window.add_toast(adw::Toast::new(&err_msg)),
            // Auth
            Action::SetClientConfig(config) => self.set_client_config(config),
            Action::Sync => self.sync().await,
            Action::Login => self.login().await,
            Action::Logout => {
                if self.logout().await.is_err() {
                    sender
                        .send(Action::Notify(gettext("Failed to logout")))
                        .await
                        .unwrap();
                }
            }
        };
        glib::ControlFlow::Continue
    }

    async fn do_article_action(&self, action: Box<ArticleAction>) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        match *action {
            ArticleAction::Add(article) => self.add_article(article),
            ArticleAction::AddMultiple(articles) => self.add_multiple_articles(articles),
            ArticleAction::Open(article) => window.load_article(article),
            ArticleAction::Delete(article) => self.delete_article(article).await,
            ArticleAction::Archive(article) => self.archive_article(article).await,
            ArticleAction::Favorite(article) => self.favorite_article(article).await,
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
                    AdwDialogExt::present(&settings_widget, Some(window));
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
                    sender.send_blocking(Action::Logout).unwrap();
                })
                .build(),
            // Sync
            gio::ActionEntry::builder("sync")
                .activate(|app: &Application, _, _| {
                    let imp = app.imp();
                    let sender = imp.sender.get().unwrap();
                    sender.send_blocking(Action::Sync).unwrap();
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
                        .send_blocking(Action::SetClientConfig(account.into()))
                        .unwrap();
                })
                .build(),
        ]);

        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);

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
                sender
                    .send_blocking(Action::SetView(View::Articles))
                    .unwrap();
                self.set_client_config(config);
            }
            _ => sender.send_blocking(Action::SetView(View::Login)).unwrap(),
        };
    }

    fn set_client_config(&self, config: Config) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            sender
                .send(Action::SetView(View::Syncing(true)))
                .await
                .unwrap();
            let logged_username = SettingsManager::string(Key::Username);

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
                        sender.send(Action::Login).await.unwrap();
                    } else {
                        sender
                            .send(Action::Notify(gettext("Failed to log in")))
                            .await
                            .unwrap();
                        sender
                            .send(Action::SetView(View::Syncing(false)))
                            .await
                            .unwrap();
                    }
                }
                Err(err) => {
                    sender
                        .send(Action::Notify(gettext("Failed to log in")))
                        .await
                        .unwrap();
                    sender
                        .send(Action::SetView(View::Syncing(false)))
                        .await
                        .unwrap();
                    error!("Failed to setup a new client from current config: {}", err);
                }
            }
        });
    }

    async fn login(&self) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap();

        self.sync().await;
        sender.send(Action::SetView(View::Articles)).await.unwrap();
    }

    async fn logout(&self) -> Result<()> {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap();

        let username = SettingsManager::string(Key::Username);
        database::wipe()?;
        window.articles_view().clear();
        SecretManager::logout(&username)?;
        SettingsManager::set_string(Key::Username, "".into());
        SettingsManager::set_integer(Key::LatestSync, 0);
        sender.send(Action::SetView(View::Login)).await.unwrap();
        Ok(())
    }

    async fn sync(&self) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        sender
            .send(Action::SetView(View::Syncing(true)))
            .await
            .unwrap();
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
                    sender.send(Action::LoadArticles(articles)).await.unwrap();
                    SettingsManager::set_integer(Key::LatestSync, now as i32);
                }
                Err(err) => error!("Failed to sync {:#?}", err),
            };
            sender
                .send(Action::SetView(View::Syncing(false)))
                .await
                .unwrap();
        });
    }

    ///   Articles
    fn add_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        window.articles_view().add(&article);
    }

    fn add_multiple_articles(&self, articles: Vec<Article>) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        window.articles_view().add_multiple(articles);
    }

    fn load_articles(&self, articles: Vec<Article>) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();

        let pool = ThreadPool::new().expect("Failed to build pool");
        articles.iter().for_each(|article| {
            let article = article.clone();
            let sender = sender.clone();
            pool.spawn_ok(async move {
                match article.insert() {
                    Ok(_) => sender
                        .send(Action::Articles(Box::new(ArticleAction::Add(
                            article.clone(),
                        ))))
                        .await
                        .unwrap(),
                    Err(_) => sender
                        .send(Action::Articles(Box::new(ArticleAction::Update(
                            article.clone(),
                        ))))
                        .await
                        .unwrap(),
                };
            });
        });
    }

    fn update_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        window.articles_view().update(&article);
    }

    async fn save_article(&self, url: Url) {
        let imp = self.imp();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!("Saving new article \"{:#?}\"", url);
        sender.send(Action::SetView(View::Articles)).await.unwrap();
        sender
            .send(Action::SetView(View::Syncing(true)))
            .await
            .unwrap();
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.save_url(url).await;
            sender
                .send(Action::SetView(View::Syncing(false)))
                .await
                .unwrap();
            sender.send(Action::Sync).await.unwrap();
        });
    }

    async fn archive_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!(
            "(Un)archiving the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        sender
            .send(Action::SetView(View::Syncing(true)))
            .await
            .unwrap();
        window.articles_view().archive(&article);

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            sender
                .send(Action::SetView(View::Syncing(false)))
                .await
                .unwrap();
        });
    }

    async fn favorite_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!(
            "(Un)favoriting the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        sender
            .send(Action::SetView(View::Syncing(true)))
            .await
            .unwrap();
        window.articles_view().favorite(&article);

        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            sender
                .send(Action::SetView(View::Syncing(false)))
                .await
                .unwrap();
        });
    }

    async fn delete_article(&self, article: Article) {
        let imp = self.imp();
        let window = imp.window.get().unwrap();
        let sender = imp.sender.get().unwrap().clone();
        let client = imp.client.get().unwrap().clone();

        info!(
            "Deleting the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        sender
            .send(Action::SetView(View::Syncing(true)))
            .await
            .unwrap();
        window.articles_view().delete(&article);

        let article_id: i32 = article.id;
        let ctx = glib::MainContext::default();
        ctx.spawn_local(async move {
            let client = client.lock().await;
            client.delete_entry(article_id).await;
            sender
                .send(Action::SetView(View::Syncing(false)))
                .await
                .unwrap();
            sender.send(Action::SetView(View::Articles)).await.unwrap();
        });
    }

    fn authors() -> Vec<&'static str> {
        env!("CARGO_PKG_AUTHORS").split(':').collect()
    }

    fn show_about_dialog(parent: &impl IsA<gtk::Widget>) {
        let dialog = adw::AboutDialog::builder()
            .application_name(glib::application_name().unwrap())
            .application_icon(config::APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/read-it-later/")
            .version(config::VERSION)
            .translator_credits(gettext("translator-credits"))
            .developers(Self::authors())
            .artists(["Tobias Bernard"])
            .build();

        dialog.present(Some(parent));
    }
}
