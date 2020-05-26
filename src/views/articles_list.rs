use crate::models::{Article, ArticleAction, ArticlesFilter, ObjectWrapper};
use crate::widgets::articles::ArticlesListWidget;
use futures::executor::ThreadPool;
use gio::prelude::*;
use glib::Sender;
use std::rc::Rc;

pub struct ArticlesListView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    model: gio::ListStore,
    filter: ArticlesFilter,
    sender: Sender<ArticleAction>,
}

impl ArticlesListView {
    pub fn new(
        name: &str,
        title: &str,
        icon: &str,
        filter: ArticlesFilter,
        client: Rc<isahc::HttpClient>,
        sender: Sender<ArticleAction>,
    ) -> Self {
        let model = gio::ListStore::new(ObjectWrapper::static_type());
        let widget = ArticlesListWidget::new(sender.clone(), client);

        let articles_view = Self {
            widget,
            model,
            name: name.to_string(),
            title: title.to_string(),
            icon: icon.to_string(),
            filter,
            sender,
        };
        articles_view.init();
        articles_view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    pub fn add(&self, article: &Article) {
        if self.index(&article).is_none() {
            let object = ObjectWrapper::new(Box::new(article));
            self.model.insert_sorted(&object, Article::compare);
        }
    }

    pub fn clear(&self) {
        self.model.remove_all();
    }

    pub fn len(&self) -> u32 {
        self.model.get_n_items()
    }

    pub fn delete(&self, article: &Article) {
        if let Some(pos) = self.index(&article) {
            self.model.remove(pos);
        }
    }

    fn init(&self) {
        let articles = Article::load(&self.filter).unwrap();
        let pool = ThreadPool::new().expect("Failed to build pool");
        let sender = self.sender.clone();

        let ctx = glib::MainContext::default();
        ctx.spawn(async move {
            let futures = async move {
                articles.into_iter().for_each(|article| {
                    send!(sender, ArticleAction::Add(article));
                })
            };
            pool.spawn_ok(futures);
        });

        self.widget.bind_model(&self.model, &self.icon, "Pretty clean!");
    }

    fn index(&self, article: &Article) -> Option<u32> {
        for i in 0..self.len() {
            let gobject = self.model.get_object(i).unwrap();
            let a: Article = gobject.downcast_ref::<ObjectWrapper>().unwrap().deserialize();

            if article.id == a.id {
                return Some(i);
            }
        }
        None
    }
}
