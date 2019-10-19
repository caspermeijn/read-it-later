use reqwest::r#async::Client;
use std::path::PathBuf;

lazy_static! {
    static ref CACHE_DIR: PathBuf = glib::get_user_cache_dir().unwrap().join("read-it-later");
}

pub struct PreviewImage {
    pub url: String,
}

impl PreviewImage {
    pub fn new(url: String) -> Self {
        let image = Self { url };
        image.download_cache();
        image
    }

    fn download_cache(&self) {
        println!("{:#?}", self.url);

        //
    }

    pub fn get_cache_path(&self) {}
}
