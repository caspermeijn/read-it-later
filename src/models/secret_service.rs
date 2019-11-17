use std::rc::Rc;
extern crate secret_service;
use secret_service::EncryptionType;
use secret_service::SecretService;
use wallabag_api::types::Config;

pub struct SecretManager {
    service: Rc<SecretService>,
}

impl SecretManager {
    pub fn new() -> Self {
        let service = Rc::new(SecretService::new(EncryptionType::Dh).unwrap());

        Self { service }
    }

    pub fn store_from_config(config: Config) {
        let service = Self::new();
        service.insert(config.username.clone(), "WALLABAG_USERNAME".to_string(), config.username.clone());
        service.insert(config.username.clone(), "WALLABAG_CLIENT_ID".to_string(), config.client_id);
        service.insert(config.username.clone(), "WALLABAG_CLIENT_SECRET".to_string(), config.client_secret);
        service.insert(config.username.clone(), "WALLABAG_PASSWORD".to_string(), config.password);
        service.insert(config.username.clone(), "WALLABAG_URL".to_string(), config.base_url);
    }

    pub fn is_logged(username: &str) -> Option<Config> {
        let service = Self::new();
        let client_id = service.retrieve(username, "WALLABAG_CLIENT_ID".to_string());
        let client_secret = service.retrieve(username, "WALLABAG_CLIENT_SECRET".to_string());
        let password = service.retrieve(username, "WALLABAG_PASSWORD".to_string());
        let base_url = service.retrieve(username, "WALLABAG_URL".to_string());

        if let (Some(client_id), Some(client_secret), Some(password), Some(base_url)) = (client_id, client_secret, password, base_url) {
            return Some(Config {
                client_id,
                client_secret,
                username: username.to_string(),
                password,
                base_url,
            });
        }
        None
    }

    fn retrieve(&self, key: &str, attribute: String) -> Option<String> {
        match self.service.search_items(vec![("wallabag_username", key), ("attr", &attribute)]) {
            Ok(search_items) => match search_items.get(0) {
                Some(item) => Some(String::from_utf8(item.get_secret().unwrap()).unwrap()),
                _ => None,
            },
            _ => None,
        }
    }

    pub fn logout(username: &str) {
        let service = Self::new();
        let collection = service.service.get_default_collection().unwrap();
        if let Ok(items) = collection.search_items(vec![("wallabag_username", &username)]) {
            items.into_iter().for_each(|item| {
                if let Err(err) = item.delete() {
                    error!("Failed to remove a secret value from the keyring {}", err);
                }
            });
        }
    }

    fn insert(&self, key: String, attr: String, val: String) {
        let collection = self.service.get_default_collection().unwrap();
        collection
            .create_item(
                &format!("Read It Later account: {}", key),
                vec![("wallabag_username", &key), ("attr", &attr)],
                &val.clone().into_bytes(),
                false,
                "text/plain",
            )
            .unwrap();
    }
}
