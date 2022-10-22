use crate::models::Article;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk_macros::spawn;

mod imp {
    use super::*;
    use crate::widgets::articles::preview::ArticlePreview;
    use glib::subclass::InitializingObject;
    use once_cell::sync::OnceCell;

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

        pub article: OnceCell<Article>,
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
        fn dispose(&self, _obj: &Self::Type) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
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
    pub fn new(article: Article) -> Self {
        let article_row: Self = glib::Object::new(&[]).unwrap();
        article_row.init(article);
        article_row
    }

    pub fn article(&self) -> &Article {
        self.imp().article.get().unwrap()
    }

    fn init(&self, article: Article) {
        let imp = self.imp();
        imp.article.set(article).unwrap();

        if let Some(title) = &self.article().title {
            imp.title_label.set_text(title);
        }

        match self.article().get_article_info(false) {
            Some(article_info) => imp.info_label.set_text(&article_info),
            None => {
                imp.info_label.hide();
            }
        };

        if let Ok(Some(preview)) = self.article().get_preview() {
            imp.content_label.set_text(&preview);
        }

        let article = self.article().clone();
        let preview_image = imp.preview_image.clone();
        spawn!(async move {
            match article.get_preview_picture().await {
                Ok(Some(pixbuf)) => preview_image.set_pixbuf(&pixbuf),
                _ => preview_image.hide(),
            };
        });
    }
}
