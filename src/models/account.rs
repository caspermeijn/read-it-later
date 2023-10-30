// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::glib;

#[derive(Debug, Default, glib::Variant)]
pub struct Account {
    pub instance_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub username: String,
    pub password: String,
}

impl From<wallabag_api::types::Config> for Account {
    fn from(config: wallabag_api::types::Config) -> Self {
        Self {
            instance_url: config.base_url,
            client_id: config.client_id,
            client_secret: config.client_secret,
            username: config.username,
            password: config.password,
        }
    }
}

impl From<Account> for wallabag_api::types::Config {
    fn from(account: Account) -> Self {
        wallabag_api::types::Config {
            base_url: account.instance_url,
            client_id: account.client_id,
            client_secret: account.client_secret,
            username: account.username,
            password: account.password,
        }
    }
}
