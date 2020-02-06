use secret_service::EncryptionType;
use secret_service::{SecretService, SsError};
use std::collections::HashMap;
use std::rc::Rc;
use wallabag_api::types::Config;

pub struct SecretManager {
    service: Rc<SecretService>,
}

impl SecretManager {
    pub fn new() -> Result<Self, SsError> {
        let service = Rc::new(SecretService::new(EncryptionType::Dh)?);
        let collection = service.get_default_collection()?;
        if collection.is_locked()? {
            collection.unlock()?;
        }
        Ok(Self { service })
    }

    pub fn logout(username: &str) -> Result<(), SsError> {
        let service = Self::new()?;

        let collection = service.service.get_default_collection()?;
        let items = collection.search_items(vec![("wallabag_username", &username)])?;

        for item in items.iter() {
            item.delete()?;
        }
        Ok(())
    }

    pub fn store_from_config(config: Config) -> Result<(), SsError> {
        let service = Self::new()?;

        let collection = service.service.get_default_collection()?;

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
            )?;
        }
        Ok(())
    }

    pub fn is_logged(username: &str) -> Result<Config, SsError> {
        let service = Self::new()?;
        let client_id = service.retrieve(username, "WALLABAG_CLIENT_ID")?;
        let client_secret = service.retrieve(username, "WALLABAG_CLIENT_SECRET")?;
        let password = service.retrieve(username, "WALLABAG_PASSWORD")?;
        let base_url = service.retrieve(username, "WALLABAG_URL")?;

        Ok(Config {
            client_id,
            client_secret,
            username: username.to_string(),
            password,
            base_url,
        })
    }

    fn retrieve(&self, key: &str, attribute: &str) -> Result<String, SsError> {
        let items = self.service.search_items(vec![("wallabag_username", key), ("attr", attribute)])?;
        if let Some(item) = items.get(0) {
            let value = item.get_secret()?;
            return Ok(String::from_utf8(value).unwrap());
        }
        Err(SsError::NoResult)
    }
}
