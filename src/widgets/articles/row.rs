use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::models::ArticleObject;

mod imp {
    use std::cell::OnceCell;

    use glib::{
        once_cell::sync::Lazy, subclass::InitializingObject, ParamSpec, ParamSpecObject, Value,
    };

    use super::*;
    use crate::widgets::articles::preview::ArticlePreview;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article_row.ui")]
    pub struct ArticleRow {
        #[template_child]
        pub preview_image: TemplateChild<ArticlePreview>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub content_label: TemplateChild<gtk::Label>,

        pub article: OnceCell<ArticleObject>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticleRow {
        const NAME: &'static str = "ArticleRow";
        type Type = super::ArticleRow;
        type ParentType = gtk::ListBoxRow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticleRow {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![ParamSpecObject::builder::<ArticleObject>("article")
                    .construct_only()
                    .build()]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "article" => self.article.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "article" => {
                    let article = value.get().unwrap();
                    self.article.set(article).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticleRow {}

    impl ListBoxRowImpl for ArticleRow {}
}

glib::wrapper! {
    pub struct ArticleRow(ObjectSubclass<imp::ArticleRow>)
        @extends gtk::Widget, gtk::ListBoxRow;
}

impl ArticleRow {
    pub fn new(article: ArticleObject) -> Self {
        glib::Object::builder().property("article", article).build()
    }

    pub fn article(&self) -> ArticleObject {
        self.property::<ArticleObject>("article")
    }
}
