use futures::executor::ThreadPool;
use gtk::{gio, glib, glib::Sender, prelude::*, subclass::prelude::*};
use gtk_macros::send;
use log::error;

use crate::models::{Article, ArticleAction, ArticleObject, ArticlesFilter};

mod imp {
    use gtk::glib::subclass::InitializingObject;

    use super::*;
    use crate::widgets::articles::ArticlesListWidget;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/articles.ui")]
    pub struct ArticlesView {
        #[template_child]
        pub stack: TemplateChild<adw::ViewStack>,
        #[template_child]
        pub unread_view: TemplateChild<ArticlesListWidget>,
        #[template_child]
        pub favorites_view: TemplateChild<ArticlesListWidget>,
        #[template_child]
        pub archive_view: TemplateChild<ArticlesListWidget>,

        pub model: gio::ListStore,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticlesView {
        const NAME: &'static str = "ArticlesView";
        type Type = super::ArticlesView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticlesView {
        fn constructed(&self) {
            self.parent_constructed();

            let filter: gtk::Filter = ArticlesFilter::favorites().into();
            let favorites_model = gtk::FilterListModel::new(Some(self.model.clone()), Some(filter));
            self.favorites_view.bind_model(&favorites_model);

            let filter: gtk::Filter = ArticlesFilter::archive().into();
            let archive_model = gtk::FilterListModel::new(Some(self.model.clone()), Some(filter));
            self.archive_view.bind_model(&archive_model);

            let filter: gtk::Filter = ArticlesFilter::unread().into();
            let unread_model = gtk::FilterListModel::new(Some(self.model.clone()), Some(filter));
            self.unread_view.bind_model(&unread_model);
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticlesView {}
}

glib::wrapper! {
    pub struct ArticlesView(ObjectSubclass<imp::ArticlesView>)
        @extends gtk::Widget;
}

impl ArticlesView {
    pub fn set_sender(&self, sender: Sender<ArticleAction>) {
        let imp = self.imp();
        imp.favorites_view.set_sender(sender.clone());
        imp.archive_view.set_sender(sender.clone());
        imp.unread_view.set_sender(sender.clone());

        let filter = ArticlesFilter::none();
        let articles = Article::load(&filter).unwrap();
        let pool = ThreadPool::new().expect("Failed to build pool");

        let ctx = glib::MainContext::default();
        ctx.spawn(async move {
            let futures = async move {
                articles.into_iter().for_each(|article| {
                    send!(sender, ArticleAction::Add(article));
                })
            };
            pool.spawn_ok(futures);
        });
    }

    pub fn add(&self, article: &Article) {
        let imp = self.imp();
        if self.index(article).is_none() {
            let object = ArticleObject::new(article.clone());
            imp.model.insert_sorted(&object, Article::compare);
        }
    }

    pub fn clear(&self) {
        let imp = self.imp();
        imp.model.remove_all();
    }

    pub fn update(&self, article: &Article) {
        self.delete(article);
        self.add(article);
    }

    pub fn delete(&self, article: &Article) {
        let imp = self.imp();
        if let Some(pos) = self.index(article) {
            imp.model.remove(pos);
        }
    }

    pub fn favorite(&self, article: &Article) {
        self.update(article);
    }

    pub fn archive(&self, article: &Article) {
        self.update(article);
    }

    fn index(&self, article: &Article) -> Option<u32> {
        let imp = self.imp();
        for i in 0..imp.model.n_items() {
            let gobject = imp.model.item(i).unwrap();
            let a = gobject.downcast_ref::<ArticleObject>().unwrap().article();

            if article.id == a.id {
                return Some(i);
            }
        }
        None
    }

    pub fn get_stack(&self) -> &adw::ViewStack {
        &self.imp().stack
    }
}
