use super::article::Article;
use super::object_wrapper::ObjectWrapper;
use crate::database;
use crate::database::Error;
use crate::diesel::query_dsl::filter_dsl::FilterDsl;
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;
use gio::prelude::*;
use glib::prelude::*;
use wallabag_api::types::EntriesFilter;

pub fn get_articles(filter: &EntriesFilter) -> Result<Vec<Article>, Error> {
    use crate::schema::*;
    let db = database::connection();
    let conn = db.get()?;

    if let Some(starred) = &filter.starred {
        return articles::table
            .filter(articles::is_starred.eq(starred))
            .load::<Article>(&conn)
            .map_err(From::from);
    } else if let Some(archived) = &filter.archive {
        return articles::table
            .filter(articles::is_archived.eq(archived))
            .load::<Article>(&conn)
            .map_err(From::from);
    } else {
        return articles::table.load::<Article>(&conn).map_err(From::from);
    }
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
        if !self.index(&article).is_some() {
            let object = ObjectWrapper::new(Box::new(article));
            self.model.insert(0, &object);
        }
    }

    fn index(&self, article: &Article) -> Option<u32> {
        for i in 0..self.get_count() {
            let gobject = self.model.get_object(i).unwrap();
            let a: Article = gobject.downcast_ref::<ObjectWrapper>().unwrap().deserialize();

            if article.id == a.id {
                return Some(i);
            }
        }
        None
    }

    pub fn get_count(&self) -> u32 {
        self.model.get_n_items()
    }
}
