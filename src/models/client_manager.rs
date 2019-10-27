use crate::models::Article;
use chrono::DateTime;
use failure::Error;
use glib::Sender;
use std::cell::RefCell;
use url::Url;
use wallabag_api::types::{Config, EntriesFilter, NewEntry, SortBy, SortOrder, User};
use wallabag_api::Client;

use crate::application::Action;

extern crate secret_service;
use secret_service::EncryptionType;
use secret_service::SecretService;

pub struct ClientManager {
    client: Option<RefCell<Client>>,
    secret_service: SecretService,
    sender: Sender<Action>,
}

impl ClientManager {
    pub fn new(sender: Sender<Action>) -> Self {
        // Check if we have a client stored in our secrets

        // Try to create a client from env variables

        // Fallback to nothing until the user logs in
        let client: Option<RefCell<Client>> = None;

        // initialize secret service (dbus connection and encryption session)
        let ss = SecretService::new(EncryptionType::Dh).unwrap();

        let manager = Self {
            client,
            secret_service: ss,
            sender,
        };
        manager
    }

    pub fn save_url(&self, url: Url) {
        if let Some(client) = &self.client {
            let new_entry = NewEntry::new_with_url(url.into_string());
            if let Ok(entry) = client.borrow_mut().create_entry(&new_entry) {
                let article = Article::from(entry);
                match article.insert() {
                    Ok(_) => self.sender.send(Action::AddArticle(article)),
                    Err(_) => self.sender.send(Action::Notify("Couldn't save the article".into())),
                };
            }
        }
    }

    pub fn sync(&self, since: DateTime<chrono::Utc>) {
        let filter = EntriesFilter {
            archive: None,
            starred: None,
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: since.timestamp(),
            public: None,
        };
        if let Some(client) = &self.client {
            if let Ok(entries) = client.borrow_mut().get_entries_with_filter(&filter) {
                for entry in entries.into_iter() {
                    let article = Article::from(entry);
                    match article.insert() {
                        Ok(_) => self.sender.send(Action::AddArticle(article)),
                        Err(_) => self.sender.send(Action::Notify("Couldn't save the article".into())),
                    };
                }
            }
        }
    }

    pub fn get_user(&mut self) -> Option<Result<User, wallabag_api::errors::ClientError>> {
        match &self.client {
            Some(client) => Some(client.borrow_mut().get_user()),
            None => None,
        }
    }

    pub fn set_username(&mut self, username: String) -> Result<User, Error> {
        if self.is_user_logged_in(username.clone()) {
            let stored_config = Config {
                client_id: self
                    .retrieve_secret(username.clone(), "WALLABAG_CLIENT_ID")
                    .expect("Failed to retrieve WALLABAG_CLIENT_ID"),
                client_secret: self
                    .retrieve_secret(username.clone(), "WALLABAG_CLIENT_SECRET")
                    .expect("Failed to retrieve WALLABAG_CLIENT_SECRET"),
                username: self
                    .retrieve_secret(username.clone(), "WALLABAG_USERNAME")
                    .expect("Failed to retrieve WALLABAG_USERNAME"),
                password: self
                    .retrieve_secret(username.clone(), "WALLABAG_PASSWORD")
                    .expect("Failed to retrieve WALLABAG_PASSWORD"),
                base_url: self.retrieve_secret(username.clone(), "WALLABAG_URL").expect("Failed to retrieve WALLABAG_URL"),
            };
            return self.set_config(stored_config);
        }
        bail!("Username not found {}", username);
    }

    pub fn set_config(&mut self, config: wallabag_api::types::Config) -> Result<User, Error> {
        let attrs = [
            ("WALLABAG_CLIENT_ID", config.client_id.clone()),
            ("WALLABAG_CLIENT_SECRET", config.client_secret.clone()),
            ("WALLABAG_USERNAME", config.username.clone()),
            ("WALLABAG_PASSWORD", config.password.clone()),
            ("WALLABAG_URL", config.base_url.clone()),
        ];

        let mut client = Client::new(config);
        if let Ok(user) = client.get_user() {
            // Store oauth required info if it wasn't saved before
            if !self.is_user_logged_in(user.username.clone()) {
                let collection = self.secret_service.get_default_collection().unwrap();

                for (attr, val) in attrs.into_iter() {
                    collection
                        .create_item(
                            &format!("Read It Later account: {}", user.username),
                            vec![("wallabag_username", &user.username), ("attr", attr)],
                            &val.clone().into_bytes(),
                            false,
                            "text/plain",
                        )
                        .unwrap();
                }
            }

            self.client.replace(RefCell::new(client));
            return Ok(user);
        }
        bail!("Failed to log in");
    }

    fn is_user_logged_in(&self, username: String) -> bool {
        let search_items = self.secret_service.search_items(vec![("wallabag_username", &username)]).unwrap();
        search_items.len() == 5
    }

    fn retrieve_secret(&self, username: String, attribute: &str) -> Result<String, Error> {
        let search_items = self
            .secret_service
            .search_items(vec![("wallabag_username", &username), ("attr", attribute)])
            .unwrap();
        let item = search_items.get(0).unwrap();
        Ok(String::from_utf8(item.get_secret().unwrap())?)
    }
}
