use crate::models::Article;
use gtk::glib;
use gtk::prelude::*;
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

    impl ObjectImpl for ArticleObject {
        fn properties() -> &'static [glib::ParamSpec] {
            use once_cell::sync::Lazy;
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecString::builder("title").read_only().build(),
                    glib::ParamSpecString::builder("preview-text").read_only().build(),
                    glib::ParamSpecString::builder("description").read_only().build(),
                    glib::ParamSpecString::builder("cover-picture-url").read_only().build(),
                ]
            });

            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let article = self.article.get().unwrap();
            match pspec.name() {
                "title" => article.title.clone().to_value(),
                "preview-text" => article.get_preview().to_value(),
                "description" => article.get_article_info(false).to_value(),
                "cover-picture-url" => article.preview_picture.clone().to_value(),
                _ => unimplemented!(),
            }
        }
    }
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
