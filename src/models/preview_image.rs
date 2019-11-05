use base64::encode;
use failure::Error;
use std::fs::File;
use std::path::PathBuf;
use std::{fs, io::Write};

lazy_static! {
    static ref CACHE_DIR: PathBuf = glib::get_user_cache_dir().unwrap().join("read-it-later");
    // List of BlackList Images or domains to never download from
    static ref BLACK_LIST: [String; 1] = ["https://s0.wp.com/i/blank.jpg".into()];
}

pub struct PreviewImage {
    pub url: String,
    cache: PathBuf,
}

impl PreviewImage {
    pub fn new(url: String) -> Self {
        let cache = PreviewImage::get_cache_of(url.clone());
        let image = Self { url, cache };
        image
    }

    pub fn get_cache_of(url: String) -> PathBuf {
        let mut cache_file = encode(&url);
        cache_file.truncate(80);
        let cache: PathBuf = CACHE_DIR.join(&cache_file);
        cache
    }

    pub async fn download(&self) -> Result<(), Error> {
        if !BLACK_LIST.contains(&self.url) && !self.cache.exists() {
            let cache_dir = &CACHE_DIR;
            if let Ok(_) = fs::create_dir_all(&cache_dir.to_str().unwrap()) {
                let resp = reqwest::get(&self.url).await?;

                let content = resp.bytes().await?;
                if !content.is_empty() {
                    let mut out = File::create(self.cache.clone())?;

                    let mut data: Vec<u8> = content.as_ref().iter().cloned().collect();
                    out.write_all(&mut data)?;
                }

                info!("Downloading preview image {} into {:#?}", self.url, self.cache);
                return Ok(());
            }
        }
        bail!("Failed to download {}", self.url);
    }
}
