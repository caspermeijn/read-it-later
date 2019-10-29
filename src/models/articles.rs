use super::article::Article;
use super::object_wrapper::ObjectWrapper;
use crate::database;
use crate::database::Error;
use gio::prelude::*;
use glib::prelude::*;
use wallabag_api::types::EntriesFilter;

pub fn get_articles(filter: &EntriesFilter) -> Result<Vec<Article>, Error> {
    use crate::schema::articles::dsl::*;
    use diesel::prelude::*;;
    let db = database::connection();

    let conn = db.get()?;

    let mut query = articles.order(published_at.desc()).into_boxed();

    if let Some(starred) = &filter.starred {
        query = query.filter(is_starred.eq(starred));
    }
    if let Some(archived) = &filter.archive {
        query = query.filter(is_archived.eq(archived));
    }
    query.get_results::<Article>(&conn).map_err(From::from)
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

    pub fn remove_article(&self, article: &Article) {
        match self.index(&article) {
            Some(pos) => self.model.remove(pos),
            None => (),
        };
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
