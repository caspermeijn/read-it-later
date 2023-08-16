use std::path::PathBuf;

use anyhow::Result;
use glib::once_cell::sync::Lazy;
use gtk::glib;
use isahc::prelude::*;
use log::info;
use sha1::{Digest, Sha1};
use url::Url;

pub static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| glib::user_cache_dir().join("read-it-later"));

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
        let cache: PathBuf = CACHE_DIR.join(&hex_hash);
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
