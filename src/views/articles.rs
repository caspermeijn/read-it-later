use futures::executor::ThreadPool;
use gtk::{gio, glib, glib::Sender, prelude::*};
use gtk_macros::send;
use log::error;

use crate::{
    models::{Article, ArticleAction, ArticleObject, ArticlesFilter},
    views::ArticlesListView,
};

#[derive(Clone, Debug)]
pub struct ArticlesView {
    pub widget: adw::ViewStack,
    unread_view: ArticlesListView,
    favorites_view: ArticlesListView,
    archive_view: ArticlesListView,
    model: gio::ListStore,
}

impl ArticlesView {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let model = gio::ListStore::new(ArticleObject::static_type());

        let favorites_view = ArticlesListView::new(
            "favorites",
            "Favorites",
            "favorites-symbolic",
            ArticlesFilter::favorites(),
            sender.clone(),
            model.clone(),
        );
        let archive_view = ArticlesListView::new(
            "archive",
            "Archive",
            "archive-symbolic",
            ArticlesFilter::archive(),
            sender.clone(),
            model.clone(),
        );
        let unread_view = ArticlesListView::new(
            "unread",
            "Unread",
            "unread-symbolic",
            ArticlesFilter::unread(),
            sender.clone(),
            model.clone(),
        );
        let widget = adw::ViewStack::builder()
            .hhomogeneous(false)
            .vhomogeneous(false)
            .build();

        let articles_view = Self {
            widget,
            archive_view,
            favorites_view,
            unread_view,
            model,
        };
        articles_view.init(sender);
        articles_view
    }

    fn init(&self, sender: Sender<ArticleAction>) {
        // Unread View
        self.widget
            .add_titled(
                self.unread_view.get_widget(),
                Some(&self.unread_view.name),
                &self.unread_view.title,
            )
            .set_icon_name(Some(&self.unread_view.icon));
        // Favorites View
        self.widget
            .add_titled(
                self.favorites_view.get_widget(),
                Some(&self.favorites_view.name),
                &self.favorites_view.title,
            )
            .set_icon_name(Some(&self.favorites_view.icon));
        // Archive View
        self.widget
            .add_titled(
                self.archive_view.get_widget(),
                Some(&self.archive_view.name),
                &self.archive_view.title,
            )
            .set_icon_name(Some(&self.archive_view.icon));

        self.widget.set_visible(true);

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
        if self.index(article).is_none() {
            let object = ArticleObject::new(article.clone());
            self.model.insert_sorted(&object, Article::compare);
        }
    }

    pub fn clear(&self) {
        self.model.remove_all();
    }

    pub fn update(&self, article: &Article) {
        self.delete(article);
        self.add(article);
    }

    pub fn delete(&self, article: &Article) {
        if let Some(pos) = self.index(article) {
            self.model.remove(pos);
        }
    }

    pub fn favorite(&self, article: &Article) {
        self.update(article);
    }

    pub fn archive(&self, article: &Article) {
        self.update(article);
    }

    fn index(&self, article: &Article) -> Option<u32> {
        for i in 0..self.model.n_items() {
            let gobject = self.model.item(i).unwrap();
            let a = gobject.downcast_ref::<ArticleObject>().unwrap().article();

            if article.id == a.id {
                return Some(i);
            }
        }
        None
    }
}
