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

        let favorites_view = ArticlesListView::new();
        favorites_view.set_property("placeholder-icon-name", "favorites-symbolic");
        favorites_view.set_sender(sender.clone());
        let filter: gtk::Filter = ArticlesFilter::favorites().into();
        let favorites_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter));
        favorites_view.bind_model(&favorites_model);

        let archive_view = ArticlesListView::new();
        archive_view.set_property("placeholder-icon-name", "archive-symbolic");
        archive_view.set_sender(sender.clone());
        let filter: gtk::Filter = ArticlesFilter::archive().into();
        let archive_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter));
        archive_view.bind_model(&archive_model);

        let unread_view = ArticlesListView::new();
        unread_view.set_property("placeholder-icon-name", "unread-symbolic");
        unread_view.set_sender(sender.clone());
        let filter: gtk::Filter = ArticlesFilter::unread().into();
        let unread_model = gtk::FilterListModel::new(Some(model.clone()), Some(filter));
        unread_view.bind_model(&unread_model);

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
            .add_titled(&self.unread_view, Some("unread"), &"Unread")
            .set_icon_name(Some("unread-symbolic"));
        // Favorites View
        self.widget
            .add_titled(&self.favorites_view, Some("favorites"), "Favorites")
            .set_icon_name(Some("favorites-symbolic"));
        // Archive View
        self.widget
            .add_titled(&self.archive_view, Some("archive"), "Archive")
            .set_icon_name(Some("archive-symbolic"));

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
