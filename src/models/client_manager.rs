use crate::models::Article;
use chrono::DateTime;
use failure::Error;
use futures::lock::Mutex;
use glib::futures::FutureExt;
use glib::Sender;
use std::sync::Arc;
use url::Url;
use wallabag_api::types::{EntriesFilter, NewEntry, SortBy, SortOrder, User};
use wallabag_api::Client;

use crate::application::Action;

#[derive(Clone, Debug)]
pub struct ClientManager {
    client: Option<Arc<Mutex<Client>>>,
    user: Option<Arc<Mutex<User>>>,
    sender: Sender<Action>,
}

impl ClientManager {
    pub fn new(sender: Sender<Action>) -> Self {
        let client: Option<Arc<Mutex<Client>>> = None;
        let user: Option<Arc<Mutex<User>>> = None;

        let manager = Self { client, sender, user };
        manager
    }

    pub async fn save_url(&self, url: Url) {
        debug!("Saving url {}", url);
        if let Some(client) = self.client.clone() {
            let sender = self.sender.clone();
            client
                .lock()
                .then(async move |mut guard| {
                    let new_entry = NewEntry::new_with_url(url.into_string());
                    if let Ok(entry) = guard.create_entry(&new_entry).await {
                        let article = Article::from(entry);
                        send!(sender, Action::AddArticle(article));
                    }
                })
                .await;
        }
    }

    pub async fn sync(&self, since: DateTime<chrono::Utc>) -> Result<(), Error> {
        let filter = EntriesFilter {
            archive: None,
            starred: None,
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: since.timestamp(),
            public: None,
        };
        if let Some(client) = self.client.clone() {
            let sender = self.sender.clone();
            let fut = client.lock().then(|mut guard| {
                async move {
                    let entries = guard.get_entries_with_filter(&filter).await;
                    match entries {
                        Ok(entries) => {
                            entries.into_iter().for_each(|entry| {
                                let article = Article::from(entry);
                                if article.insert().is_ok() {
                                    send!(sender, Action::AddArticle(article));
                                }
                            });
                        }
                        Err(_) => (),
                    };
                }
            });
            fut.await;
            return Ok(());
        }
        bail!("No client set yet");
    }

    pub fn get_user(&self) -> Option<Arc<Mutex<User>>> {
        let user = self.user.clone();
        user
    }

    pub async fn fetch_user(&self) -> Result<User, Error> {
        if let Some(client) = self.client.clone() {
            let fut = client.lock().then(|mut target| {
                async move {
                    let user = target.get_user().await?;
                    Ok(user) as Result<User, Error>
                }
            });
            return Ok(fut.await?);
        }
        bail!("No client set yet");
    }

    pub async fn set_config(&mut self, config: wallabag_api::types::Config) -> Result<(), Error> {
        let client = Client::new(config);
        self.client = Some(Arc::new(Mutex::new(client)));
        if let Ok(user) = self.fetch_user().await {
            self.user.replace(Arc::new(Mutex::new(user)));
        }
        Ok(())
    }
}
