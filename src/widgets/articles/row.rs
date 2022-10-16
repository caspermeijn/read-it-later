use crate::models::Article;
use crate::models::ArticleAction;
use crate::widgets::articles::preview::ArticlePreview;
use glib::subclass::InitializingObject;
use glib::Object;
use gtk::glib::clone;
use gtk::glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk_macros::{send, spawn};
use log::error;
use once_cell::sync::OnceCell;
use std::rc::Rc;

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article_row.ui")]
    pub struct ArticleRow {
        #[template_child]
        pub article_row: TemplateChild<gtk::ListBoxRow>,
        #[template_child]
        pub preview_image: TemplateChild<ArticlePreview>,
        #[template_child]
        pub title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub info_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub content_label: TemplateChild<gtk::Label>,

        pub article: OnceCell<Article>,
        pub sender: OnceCell<Sender<ArticleAction>>,
        pub client: OnceCell<Rc<isahc::HttpClient>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticleRow {
        const NAME: &'static str = "ArticleRow";
        type Type = super::ArticleRow;
        type ParentType = gtk::Widget;

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
}

glib::wrapper! {
    pub struct ArticleRow(ObjectSubclass<imp::ArticleRow>)
        @extends gtk::Widget;
}

impl ArticleRow {
    pub fn new(article: Article, client: Rc<isahc::HttpClient>, sender: Sender<ArticleAction>) -> Self {
        let article_row: Self = Object::new(&[]).unwrap();
        article_row.init(article, client, sender);
        article_row
    }

    fn article(&self) -> &Article {
        self.imp().article.get().unwrap()
    }

    fn init(&self, article: Article, client: Rc<isahc::HttpClient>, sender: Sender<ArticleAction>) {
        self.imp().article.set(article).unwrap();
        self.imp().client.set(client).unwrap();
        self.imp().sender.set(sender).unwrap();

        let event_controller = gtk::GestureClick::new();
        self.imp().article_row.add_controller(&event_controller);
        event_controller.connect_pressed(clone!(@strong self as article_row => move |_, _, _, _| {
            let sender = article_row.imp().sender.get().unwrap();
            let article = article_row.imp().article.get().unwrap();
            send!(sender, ArticleAction::Open(article.clone()));
        }));

        if let Some(title) = &self.article().title {
            self.imp().title_label.set_text(title);
        }

        match self.article().get_article_info(false) {
            Some(article_info) => self.imp().info_label.set_text(&article_info),
            None => {
                self.imp().info_label.hide();
            }
        };

        if let Ok(Some(preview)) = self.article().get_preview() {
            self.imp().content_label.set_text(&preview);
        }

        let article = self.article().clone();
        let preview_image = self.imp().preview_image.clone();
        let client = self.imp().client.get().unwrap().clone();
        spawn!(async move {
            match article.get_preview_picture(client).await {
                Ok(Some(pixbuf)) => preview_image.set_pixbuf(&pixbuf),
                _ => preview_image.hide(),
            };
        });
    }

    pub fn get_list_box_row(&self) -> &gtk::ListBoxRow {
        &self.imp().article_row
    }
}
