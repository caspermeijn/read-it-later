use crypto::digest::Digest;
use crypto::sha1::Sha1;
use failure::Error;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io::Write};
use url::Url;

lazy_static! {
    static ref CACHE_DIR: PathBuf = glib::get_user_cache_dir().unwrap().join("read-it-later");
}

pub struct PreviewImage {
    pub url: Url,
    cache: PathBuf,
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

    pub async fn download(&self) -> Result<(), Error> {
        if !self.cache.exists() {
            let cache_dir = &CACHE_DIR;
            fs::create_dir_all(&cache_dir.to_str().unwrap())?;

            if let Ok(mut resp) = surf::get(&self.url).await {
                let content = resp.body_bytes().await?;
                if !content.is_empty() {
                    let mut out = File::create(self.cache.clone())?;
                    out.write_all(&content)?;
                }
            }

            info!("Downloading preview image {} into {:#?}", self.url, self.cache);
        }
        Ok(())
    }
}
