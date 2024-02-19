// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2020 Julian Hofer <julian.git@mailbox.org>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{path::PathBuf, sync::OnceLock};

use anyhow::Result;
use gtk::glib;
use isahc::prelude::*;
use log::info;
use sha1::{Digest, Sha1};
use url::Url;

fn cache_dir() -> &'static PathBuf {
    static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();
    CACHE_DIR.get_or_init(|| {
        let cache_dir = glib::user_cache_dir().join("read-it-later");
        std::fs::create_dir_all(&cache_dir).unwrap();
        cache_dir
    })
}

pub struct PreviewImage {
    pub url: Url,
    pub cache: PathBuf,
}

impl PreviewImage {
    pub fn new(url: Url) -> Self {
        let cache = PreviewImage::get_cache_of(&String::from(url.clone()));
        Self { url, cache }
    }

    pub fn get_cache_of(path: &str) -> PathBuf {
        let hash = Sha1::digest(path);
        let hex_hash = base16ct::lower::encode_string(&hash);
        let cache: PathBuf = cache_dir().join(hex_hash);
        cache
    }

    pub fn exists(&self) -> bool {
        self.cache.exists()
    }

    pub async fn download(&self) -> Result<()> {
        if let Ok(mut resp) = isahc::get_async(&self.url.to_string()).await {
            info!(
                "Downloading preview image {} into {:#?}",
                self.url, self.cache
            );
            let body = resp.bytes().await?;
            async_std::fs::write(self.cache.clone(), body).await?;
        }
        Ok(())
    }
}
