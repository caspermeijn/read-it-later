use crate::models::Article;
use gtk::glib;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;

    use once_cell::sync::OnceCell;

    #[derive(Default)]
    pub struct ArticleObject {
        pub article: OnceCell<Article>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticleObject {
        const NAME: &'static str = "ArticleObject";
        type Type = super::ArticleObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ArticleObject {}
}

glib::wrapper! {
    pub struct ArticleObject(ObjectSubclass<imp::ArticleObject>);
}

impl ArticleObject {
    pub fn new(article: Article) -> Self {
        let obj: Self = glib::Object::new(&[]);
        obj.imp().article.set(article).unwrap();
        obj
    }

    pub fn article(&self) -> &Article {
        self.imp().article.get().unwrap()
    }
}
