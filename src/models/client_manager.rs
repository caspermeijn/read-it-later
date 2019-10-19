use crate::models::Article;
use failure::Error;
use wallabag_api::types::{Config, User};
use wallabag_api::Client;

extern crate secret_service;
use secret_service::EncryptionType;
use secret_service::SecretService;

pub struct ClientManager {
    client: Option<Client>,
    secret_service: SecretService,
}

impl ClientManager {
    pub fn new() -> Self {
        // Check if we have a client stored in our secrets

        // Try to create a client from env variables

        // Fallback to nothing until the user logs in
        let client = None;

        // initialize secret service (dbus connection and encryption session)
        let ss = SecretService::new(EncryptionType::Dh).unwrap();

        let manager = Self { client, secret_service: ss };
        manager
    }

    pub fn sync(&mut self) {
        match self.client.as_mut() {
            Some(client) => {
                info!("Fetching the latest entries");
                if let Ok(entries) = client.get_entries() {
                    println!("{:#?}", entries);
                    for entry in entries.into_iter() {
                        let article = Article::from(entry);
                        println!("{:#?}", article);
                        article.insert();
                    }
                }
            }
            None => warn!("You have to be logged in in order to sync"),
        }
    }

    pub fn set_username(&mut self, username: String) -> Result<User, Error> {
        println!("{:#?}", username);
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
            // get default collection
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

            self.client.replace(client);
            return Ok(user);
        }
        bail!("Failed to log in");
    }

    fn is_user_logged_in(&self, username: String) -> bool {
        let search_items = self.secret_service.search_items(vec![("wallabag_username", &username)]).unwrap();
        println!("{:#?}", search_items);
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
