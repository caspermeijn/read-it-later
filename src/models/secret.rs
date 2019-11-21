use anyhow::Result;
use secret_service::EncryptionType;
use secret_service::SecretService;
use std::collections::HashMap;
use std::rc::Rc;
use wallabag_api::types::Config;

pub struct SecretManager {
    service: Rc<SecretService>,
}

impl SecretManager {
    pub fn new() -> Self {
        let service = Rc::new(SecretService::new(EncryptionType::Dh).unwrap());

        Self { service }
    }

    pub fn logout(username: &str) -> Result<()> {
        let service = Self::new();
        service.service.get_default_collection().and_then(|collection| {
            collection.search_items(vec![("wallabag_username", &username)]).and_then(|items| {
                items.iter().for_each(|item| {
                    item.delete();
                });
                Ok(())
            })
        });
        Ok(())
    }

    pub fn store_from_config(config: Config) -> Result<()> {
        let service = Self::new();
        service.service.get_default_collection().and_then(|collection| {
            let username = config.username.clone();
            let mut secret_config = HashMap::new();
            secret_config.insert("WALLABAG_USERNAME".to_string(), config.username.clone());
            secret_config.insert("WALLABAG_CLIENT_ID".to_string(), config.client_id);
            secret_config.insert("WALLABAG_CLIENT_SECRET".to_string(), config.client_secret);
            secret_config.insert("WALLABAG_PASSWORD".to_string(), config.password);
            secret_config.insert("WALLABAG_URL".to_string(), config.base_url);

            for (key, val) in secret_config.iter() {
                collection.create_item(
                    &format!("Read It Later account: {}", username),
                    vec![("wallabag_username", &username), ("attr", &key)],
                    &val.clone().into_bytes(),
                    false,
                    "text/plain",
                );
            }
            Ok(())
        });
        Ok(())
    }

    pub fn is_logged(username: &str) -> Option<Config> {
        let service = Self::new();
        let client_id = service.retrieve(username, "WALLABAG_CLIENT_ID");
        let client_secret = service.retrieve(username, "WALLABAG_CLIENT_SECRET");
        let password = service.retrieve(username, "WALLABAG_PASSWORD");
        let base_url = service.retrieve(username, "WALLABAG_URL");

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

    fn retrieve(&self, key: &str, attribute: &str) -> Option<String> {
        match self.service.search_items(vec![("wallabag_username", key), ("attr", attribute)]) {
            Ok(search_items) => match search_items.get(0) {
                Some(item) => Some(String::from_utf8(item.get_secret().unwrap()).unwrap()),
                _ => None,
            },
            _ => None,
        }
    }
}
