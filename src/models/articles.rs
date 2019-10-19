use super::article::Article;
use super::object_wrapper::ObjectWrapper;
use crate::database;
use crate::database::Error;
use crate::diesel::query_dsl::filter_dsl::FilterDsl;
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use gio::prelude::*;
use glib::prelude::*;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

pub fn get_articles(filter: &EntriesFilter) -> Result<Vec<Article>, Error> {
    use crate::schema::*;
    let db = database::connection();
    let conn = db.get()?;

    let mut items = articles::table;
    if let Some(starred) = &filter.starred {
        items.filter(articles::is_starred.eq(starred));
    }
    if let Some(archived) = &filter.archive {
        items.filter(articles::is_archived.eq(archived));
    }
    items.load::<Article>(&conn).map_err(From::from)
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
            for article in articles.into_iter() {
                self.add_article(&article);
            }
        }
    }

    pub fn add_article(&self, article: &Article) {
        let object = ObjectWrapper::new(Box::new(article));
        self.model.insert(0, &object);
    }

    pub fn get_count(&self) -> u32 {
        self.model.get_n_items()
    }
}
