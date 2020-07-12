use anyhow::Result;
use crypto::digest::Digest;
use crypto::sha1::Sha1;
use isahc::ResponseExt;
use std::path::PathBuf;
use std::rc::Rc;
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

    pub async fn download(&self, client: Rc<isahc::HttpClient>) -> Result<()> {
        if let Ok(mut resp) = client.get_async(&self.url.to_string()).await {
            info!("Downloading preview image {} into {:#?}", self.url, self.cache);
            resp.copy_to_file(self.cache.clone())?;
        }
        Ok(())
    }
}
