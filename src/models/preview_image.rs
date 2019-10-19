use base64::{decode, encode};
use reqwest::r#async::Client;
use std::fs::File;
use std::io;
use std::path::PathBuf;

lazy_static! {
    static ref CACHE_DIR: PathBuf = glib::get_user_cache_dir().unwrap().join("read-it-later");
}

pub enum PreviewImageType {
    Small,
    Large,
}

pub struct PreviewImage {
    pub url: String,
    cache: PathBuf,
}

impl PreviewImage {
    pub fn new(url: String) -> Self {
        let cache: PathBuf = CACHE_DIR.join(encode(&url));

        let image = Self { url, cache };
        image.download_cache();
        image
    }

    fn download_cache(&self) {
        if !self.cache.exists() {
            let mut resp = reqwest::get(&self.url).expect("request failed");
            let mut out = File::create(self.cache.clone()).expect("failed to create file");
            io::copy(&mut resp, &mut out).expect("failed to copy content");

            info!("Downloading preview image {} into {:#?}", self.url, self.cache);
        }
    }

    pub fn get_cache_path(&self) -> PathBuf {
        self.cache.clone()
    }
}
