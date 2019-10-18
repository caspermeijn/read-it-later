use crate::models::Article;
use std::cell::RefCell;
use wallabag_api::Client;

pub struct ClientManager {
    client: Option<Client>,
}

impl ClientManager {
    pub fn new() -> Self {
        // Check if we have a client stored in our secrets

        // Try to create a client from env variables

        // Fallback to nothing until the user logs in
        let client = None;

        let manager = Self { client };
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

    pub fn set_config(&mut self, config: wallabag_api::types::Config) {
        let client = Client::new(config);
        self.client.replace(client);
        self.sync();
    }
}
