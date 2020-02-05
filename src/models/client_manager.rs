use async_std::sync::{Arc, Mutex};
use chrono::DateTime;
use failure::Error;
use glib::Sender;
use url::Url;
use wallabag_api::types::{EntriesFilter, NewEntry, PatchEntry, SortBy, SortOrder, User};
use wallabag_api::Client;

use crate::application::Action;
use crate::models::{Article, ArticleAction};

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

        Self { client, sender, user }
    }

    pub async fn update_entry(&self, entry_id: i32, patch: PatchEntry) {
        debug!("[Client] Updating entry {}", entry_id);
        if let Some(client) = self.client.clone() {
            let mut client = client.lock().await;
            if client.update_entry(entry_id, &patch).await.is_err() {
                warn!("[Client] Failed to update the entry {}", entry_id);
            }
        }
    }

    pub async fn delete_entry(&self, entry_id: i32) {
        debug!("[Client] Removing entry {}", entry_id);
        if let Some(client) = self.client.clone() {
            let mut client = client.lock().await;
            if client.delete_entry(entry_id).await.is_err() {
                warn!("[Client] Failed to delete the entry {}", entry_id);
            }
        }
    }

    pub async fn save_url(&self, url: Url) {
        debug!("[Client] Saving url {}", url);
        if let Some(client) = self.client.clone() {
            let mut client = client.lock().await;
            let new_entry = NewEntry::new_with_url(url.into_string());
            if let Ok(entry) = client.create_entry(&new_entry).await {
                let article = Article::from(entry);
                send!(self.sender, Action::Articles(Box::new(ArticleAction::Add(article))));
            }
        }
    }

    pub async fn sync(&self, since: DateTime<chrono::Utc>) -> Result<Vec<Article>, Error> {
        let filter = EntriesFilter {
            archive: None,
            starred: None,
            sort: SortBy::Created,
            order: SortOrder::Asc,
            tags: vec![],
            since: since.timestamp(),
            public: None,
        };
        if let Some(client) = self.client.clone() {
            let mut client = client.lock().await;
            let entries = client.get_entries_with_filter(&filter).await?;
            let articles = entries.into_iter().map(Article::from).collect::<Vec<Article>>();
            return Ok(articles);
        }
        bail!("No client set yet")
    }

    pub fn get_user(&self) -> Option<Arc<Mutex<User>>> {
        self.user.clone()
    }

    pub async fn fetch_user(&self) -> Result<User, Error> {
        if let Some(client) = self.client.clone() {
            let mut client = client.lock().await;
            let user = client.get_user().await?;
            return Ok(user);
        }
        bail!("No client set yet");
    }

    pub async fn set_config(&mut self, config: wallabag_api::types::Config) -> Result<(), Error> {
        let client = Client::new(config);
        self.client = Some(Arc::new(Mutex::new(client)));
        let user = self.fetch_user().await?;
        self.user.replace(Arc::new(Mutex::new(user)));
        Ok(())
    }
}
