use std::path::PathBuf;

use anyhow::Result;
use crypto::{digest::Digest, sha1::Sha1};
use gtk::glib;
use isahc::prelude::*;
use log::info;
use once_cell::sync::Lazy;
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
        let mut hasher = Sha1::new();
        hasher.input_str(path);
        let cache: PathBuf = CACHE_DIR.join(&hasher.result_str());
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
