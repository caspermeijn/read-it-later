use std::cell::RefCell;

use adw::{prelude::*, subclass::prelude::*};
use anyhow::Result;
use async_std::sync::{Arc, Mutex};
use chrono::{TimeZone, Utc};
use futures::executor::ThreadPool;
use gettextrs::gettext;
use glib::{clone, Receiver, Sender};
use gtk::{gio, glib};
use gtk_macros::{action, send, spawn};
use log::{error, info};
use once_cell::sync::OnceCell;
use url::Url;
use wallabag_api::types::Config;

use self::config::{APP_ID, VERSION};
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
    pub struct Application {
        pub window: Window,
        pub sender: Sender<Action>,
        pub receiver: RefCell<Option<Receiver<Action>>>,
        pub client: Arc<Mutex<ClientManager>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Application {
        const NAME: &'static str = "Application";
        type Type = super::Application;
        type ParentType = adw::Application;

        fn new() -> Self {
            let (sender, r) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
            let receiver = RefCell::new(Some(r));
            let client = Arc::new(Mutex::new(ClientManager::new(sender.clone())));

            let window = Window::new(sender.clone());

            Self {
                sender,
                receiver,
                client,
                window,
            }
        }
    }

    impl ObjectImpl for Application {}

    impl WidgetImpl for Application {}

    impl ApplicationImpl for Application {
        fn activate(&self) {
            self.parent_activate();

            let app = self.obj();

            let receiver = app.imp().receiver.borrow_mut().take().unwrap();
            receiver.attach(
                None,
                clone!(@strong app => move |action| app.do_action(action)),
            );

            app.setup_gactions();
            app.init_client();

            app.add_window(&self.window);
            self.window.present();
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
            .property("application-id", Some(config::APP_ID))
            .property("resource-base-path", Some("/com/belmoussaoui/ReadItLater"))
            .build();

        app.run()
    }

    fn do_action(&self, action: Action) -> glib::Continue {
        let imp = self.imp();
        match action {
            // Articles
            Action::Articles(article_action) => self.do_article_action(article_action),
            Action::SaveArticle(url) => self.save_article(url),
            Action::LoadArticles(articles) => self.load_articles(articles),
            // UI
            Action::SetView(view) => imp.window.set_view(view),
            Action::PreviousView => imp.window.previous_view(),
            Action::Notify(err_msg) => imp.window.add_toast(adw::Toast::new(&err_msg)),
            // Auth
            Action::SetClientConfig(config) => self.set_client_config(config),
            Action::Sync => self.sync(),
            Action::Login => self.login(),
            Action::Logout => {
                if self.logout().is_err() {
                    send!(imp.sender, Action::Notify(gettext("Failed to logout")));
                }
            }
        };
        glib::Continue(true)
    }

    fn do_article_action(&self, action: Box<ArticleAction>) {
        let imp = self.imp();
        match *action {
            ArticleAction::Add(article) => self.add_article(article),
            ArticleAction::Open(article) => imp.window.load_article(article),
            ArticleAction::Delete(article) => self.delete_article(article),
            ArticleAction::Archive(article) => self.archive_article(article),
            ArticleAction::Favorite(article) => self.favorite_article(article),
            ArticleAction::Update(article) => self.update_article(article),
        };
    }

    fn setup_gactions(&self) {
        let imp = self.imp();
        let window = imp.window.clone();
        let sender = imp.sender.clone();
        let client = imp.client.clone();

        // Quit
        action!(
            self,
            "quit",
            clone!(@strong self as app => move |_, _| {
                app.quit();
            })
        );
        // Settings
        action!(
            self,
            "settings",
            clone!(@strong window, @strong client =>  move |_, _| {
                let settings_widget = SettingsWidget::new(client.clone());
                settings_widget.set_transient_for(Some(&window));
                settings_widget.present();
            })
        );
        // About
        action!(
            self,
            "about",
            clone!(@strong window => move |_, _| {
                Application::show_about_dialog(&window);
            })
        );
        action!(
            self,
            "new-article",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::SetView(View::NewArticle));
            })
        );
        action!(
            self,
            "logout",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::Logout);
            })
        );
        action!(
            self,
            "sync",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::Sync);
            })
        );
        action!(
            self,
            "login",
            Some(&crate::models::Account::static_variant_type()),
            clone!(@strong sender => move |_, parameter| {
                let account: Account = parameter.unwrap().get().unwrap();
                send!(sender, Action::SetClientConfig(account.into()));
            })
        );

        self.set_accels_for_action("win.show-help-overlay", &["<primary>question"]);
        self.set_accels_for_action("win.previous", &["Escape"]);

        self.set_accels_for_action("app.quit", &["<primary>q"]);
        self.set_accels_for_action("app.settings", &["<primary>comma"]);
        self.set_accels_for_action("app.new-article", &["<primary>N"]);
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

        let username = SettingsManager::string(Key::Username);
        match SecretManager::is_logged(&username) {
            Ok(config) => {
                send!(imp.sender, Action::SetView(View::Articles));
                self.set_client_config(config);
            }
            _ => send!(imp.sender, Action::SetView(View::Login)),
        };
    }

    fn set_client_config(&self, config: Config) {
        let imp = self.imp();
        let client = imp.client.clone();
        let sender = imp.sender.clone();

        send!(imp.sender, Action::SetView(View::Syncing(true)));
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
                        send!(sender, Action::Notify(gettext("Failed to log in")));
                        send!(sender, Action::SetView(View::Syncing(false)));
                    }
                }
                Err(err) => {
                    send!(sender, Action::Notify(gettext("Failed to log in")));
                    send!(sender, Action::SetView(View::Syncing(false)));
                    error!("Failed to setup a new client from current config: {}", err);
                }
            }
        });
    }

    fn login(&self) {
        let imp = self.imp();

        self.sync();
        send!(imp.sender, Action::SetView(View::Articles));
    }

    fn logout(&self) -> Result<()> {
        let imp = self.imp();

        let username = SettingsManager::string(Key::Username);
        database::wipe()?;
        imp.window.articles_view().clear();
        if SecretManager::logout(&username).is_ok() {
            SettingsManager::set_string(Key::Username, "".into());
            SettingsManager::set_integer(Key::LatestSync, 0);
        }
        send!(imp.sender, Action::SetView(View::Login));
        Ok(())
    }

    fn sync(&self) {
        let imp = self.imp();
        let client = imp.client.clone();
        let sender = imp.sender.clone();

        send!(sender, Action::SetView(View::Syncing(true)));
        let mut since = Utc.timestamp_opt(0, 0).unwrap();
        let last_sync = SettingsManager::integer(Key::LatestSync);
        if last_sync != 0 {
            since = Utc.timestamp_opt(last_sync.into(), 0).unwrap();
        }
        info!("Last sync was at {}", since);

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

    ///   Articles

    fn add_article(&self, article: Article) {
        let imp = self.imp();
        imp.window.articles_view().add(&article);
    }

    fn load_articles(&self, articles: Vec<Article>) {
        let imp = self.imp();
        let sender = imp.sender.clone();

        let pool = ThreadPool::new().expect("Failed to build pool");
        spawn!(async move {
            let futures = async move {
                articles.iter().for_each(|article| {
                    match article.insert() {
                        Ok(_) => send!(
                            sender,
                            Action::Articles(Box::new(ArticleAction::Add(article.clone())))
                        ),
                        Err(_) => send!(
                            sender,
                            Action::Articles(Box::new(ArticleAction::Update(article.clone())))
                        ),
                    };
                })
            };
            pool.spawn_ok(futures);
        });
    }

    fn update_article(&self, article: Article) {
        let imp = self.imp();
        imp.window.articles_view().update(&article);
    }

    fn save_article(&self, url: Url) {
        let imp = self.imp();
        let client = imp.client.clone();
        let sender = imp.sender.clone();

        info!("Saving new article \"{:#?}\"", url);
        send!(sender, Action::PreviousView);
        send!(sender, Action::SetView(View::Syncing(true)));
        spawn!(async move {
            let client = client.lock().await;
            client.save_url(url).await;
            send!(sender, Action::SetView(View::Syncing(false)));
            send!(sender, Action::Sync);
        });
    }

    fn archive_article(&self, article: Article) {
        let imp = self.imp();
        let client = imp.client.clone();
        let sender = imp.sender.clone();

        info!(
            "(Un)archiving the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        send!(imp.sender, Action::SetView(View::Syncing(true)));
        imp.window.articles_view().archive(&article);

        spawn!(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            send!(sender, Action::SetView(View::Syncing(false)));
        });
    }

    fn favorite_article(&self, article: Article) {
        let imp = self.imp();
        let client = imp.client.clone();
        let sender = imp.sender.clone();

        info!(
            "(Un)favoriting the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        send!(sender, Action::SetView(View::Syncing(true)));
        imp.window.articles_view().favorite(&article);

        spawn!(async move {
            let client = client.lock().await;
            client.update_entry(article.id, article.get_patch()).await;
            send!(sender, Action::SetView(View::Syncing(false)));
        });
    }

    fn delete_article(&self, article: Article) {
        let imp = self.imp();
        let client = imp.client.clone();
        let sender = imp.sender.clone();

        info!(
            "Deleting the article \"{:#?}\" with ID: {}",
            article.title, article.id
        );
        send!(sender, Action::SetView(View::Syncing(true)));
        imp.window.articles_view().delete(&article);

        let article_id: i32 = article.id;
        spawn!(async move {
            let client = client.lock().await;
            client.delete_entry(article_id).await;
            send!(sender, Action::SetView(View::Syncing(false)));
            send!(sender, Action::PreviousView);
        });
    }

    fn authors() -> Vec<&'static str> {
        env!("CARGO_PKG_AUTHORS").split(":").collect()
    }

    fn show_about_dialog(parent: &impl IsA<gtk::Window>) {
        let dialog = adw::AboutWindow::builder()
            .application_name(glib::application_name().unwrap())
            .application_icon(APP_ID)
            .license_type(gtk::License::Gpl30)
            .website("https://gitlab.gnome.org/World/read-it-later/")
            .version(VERSION)
            .translator_credits(&gettext("translator-credits"))
            .developers(Self::authors())
            .artists(["Tobias Bernard"])
            .transient_for(parent)
            .build();

        dialog.present();
    }
}
