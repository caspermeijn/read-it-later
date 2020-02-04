use crypto::digest::Digest;
use crypto::sha1::Sha1;
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
        let cache = PreviewImage::get_cache_of(&url);
        let image = Self { url, cache };
        image
    }

    pub fn get_cache_of(url: &str) -> PathBuf {
        let mut hasher = Sha1::new();
        hasher.input_str(url);

        let cache: PathBuf = CACHE_DIR.join(&hasher.result_str());
        cache
    }

    pub async fn download(&self) -> Result<(), Error> {
        if !BLACK_LIST.contains(&self.url) && !self.cache.exists() {
            let cache_dir = &CACHE_DIR;
            fs::create_dir_all(&cache_dir.to_str().unwrap())?;

            if let Ok(mut resp) = surf::get(&self.url).await {
                let mut content = resp.body_bytes().await?;
                if !content.is_empty() {
                    let mut out = File::create(self.cache.clone())?;
                    out.write_all(&mut content)?;
                }
            }

            info!("Downloading preview image {} into {:#?}", self.url, self.cache);
            return Ok(());
        }
        bail!("Preview Image blacklisted or already exists")
    }
}
