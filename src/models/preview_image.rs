use async_std::fs::File;
use async_std::prelude::*;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use failure::Error;
use std::path::PathBuf;
use url::Url;

lazy_static! {
    pub static ref CACHE_DIR: PathBuf = glib::get_user_cache_dir().unwrap().join("read-it-later");
}

pub struct PreviewImage {
    pub url: Url,
    pub cache: PathBuf,
}

impl PreviewImage {
    pub fn new(url: Url) -> Self {
        let cache = PreviewImage::get_cache_of(&url.clone().into_string());
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

    pub async fn download(&self) -> Result<(), Error> {
        if let Ok(mut resp) = surf::get(&self.url).await {
            info!("Downloading preview image {} into {:#?}", self.url, self.cache);
            let content = resp.body_bytes().await?;
            if !content.is_empty() {
                let mut out = File::create(self.cache.clone()).await?;
                out.write_all(&content).await?;
            }
        }
        Ok(())
    }
}
