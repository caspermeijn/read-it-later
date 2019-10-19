use gio::prelude::*;
use glib::prelude::*;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use super::article::Article;
use super::object_wrapper::ObjectWrapper;
use crate::database;
use crate::database::Error;
use crate::diesel::RunQueryDsl;

pub fn get_articles(filter: &EntriesFilter) -> Result<Vec<Article>, Error> {
    use crate::schema::articles::dsl::*;
    let db = database::connection();
    let conn = db.get()?;

    articles.load::<Article>(&conn).map_err(From::from)
}

pub struct ArticlesModel {
    pub model: gio::ListStore,
    filter: EntriesFilter,
}

impl ArticlesModel {
    pub fn new(filter: EntriesFilter) -> Self {
        let gio_model = gio::ListStore::new(ObjectWrapper::static_type());
        let model = Self { model: gio_model, filter };
        model.init();
        model
    }

    fn init(&self) {
        // fill in the articles from the database
        if let Ok(articles) = get_articles(&self.filter) {
            let mut i = 0;
            for article in articles.into_iter() {
                self.add_article(&article);
                i = i + 1;
                if i == 15 {
                    break;
                }
            }
        }
    }

    pub fn add_article(&self, article: &Article) {
        let object = ObjectWrapper::new(Box::new(article));
        self.model.append(&object);
    }

    pub fn get_count(&self) -> u32 {
        self.model.get_n_items()
    }
}
