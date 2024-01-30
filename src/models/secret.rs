// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use secret_service::{blocking::SecretService, EncryptionType, Error};
use wallabag_api::types::Config;

pub struct SecretManager<'a> {
    service: SecretService<'a>,
}

impl SecretManager<'_> {
    pub fn new() -> Result<Self, Error> {
        let service = SecretService::connect(EncryptionType::Dh)?;
        {
            let collection = service.get_default_collection()?;
            if collection.is_locked()? {
                collection.unlock()?;
            }
        }
        Ok(Self { service })
    }

    pub fn logout(username: &str) -> Result<(), Error> {
        let service = Self::new()?;

        let collection = service.service.get_default_collection()?;
        let items = collection.search_items(HashMap::from([("wallabag_username", username)]))?;

        for item in items.iter() {
            item.delete()?;
        }
        Ok(())
    }

    pub fn store_from_config(config: Config) -> Result<(), Error> {
        let service = Self::new()?;

        let collection = service.service.get_default_collection()?;

        let username = config.username.as_str();
        let mut secret_config = HashMap::new();
        secret_config.insert("WALLABAG_USERNAME", config.username.clone());
        secret_config.insert("WALLABAG_CLIENT_ID", config.client_id);
        secret_config.insert("WALLABAG_CLIENT_SECRET", config.client_secret);
        secret_config.insert("WALLABAG_PASSWORD", config.password);
        secret_config.insert("WALLABAG_URL", config.base_url);

        for (&key, val) in secret_config.iter() {
            collection.create_item(
                &format!("Read It Later account: {}", username),
                HashMap::from([("wallabag_username", username), ("attr", key)]),
                &val.clone().into_bytes(),
                false,
                "text/plain",
            )?;
        }
        Ok(())
    }

    pub fn is_logged(username: &str) -> Result<Config, Error> {
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

    fn retrieve(&self, key: &str, attribute: &str) -> Result<String, Error> {
        let attributes = HashMap::from([("wallabag_username", key), ("attr", attribute)]);
        let items = self.service.search_items(attributes)?;
        if let Some(item) = items.unlocked.first() {
            let value = item.get_secret()?;
            return Ok(String::from_utf8(value).unwrap());
        }
        Err(Error::NoResult)
    }
}
