use failure::Error;
use glib::Sender;
use gtk::prelude::*;

use crate::models::{Article, ArticleAction, ArticlesFilter};
use crate::views::ArticlesListView;

pub struct ArticlesView {
    pub widget: gtk::Stack,
    unread_view: ArticlesListView,
    favorites_view: ArticlesListView,
    archive_view: ArticlesListView,
    sender: Sender<ArticleAction>,
}

impl ArticlesView {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let favorites_view = ArticlesListView::new(
            "favorites",
            "Favorites",
            "favorites-symbolic",
            ArticlesFilter::favorites(),
            sender.clone(),
        );
        let archive_view = ArticlesListView::new("archive", "Archive", "archive-symbolic", ArticlesFilter::archive(), sender.clone());
        let unread_view = ArticlesListView::new("unread", "Unread", "unread-symbolic", ArticlesFilter::unread(), sender.clone());
        let widget = gtk::Stack::new();

        let articles_view = Self {
            widget,
            sender,
            archive_view,
            favorites_view,
            unread_view,
        };
        articles_view.init();
        articles_view
    }

    fn init(&self) {
        self.widget.set_homogeneous(false);

        // Unread View
        self.widget
            .add_titled(&self.unread_view.get_widget(), &self.unread_view.name, &self.unread_view.title);
        self.widget
            .set_child_icon_name(&self.unread_view.get_widget(), Some(&self.unread_view.icon));
        // Favorites View
        self.widget.add_titled(
            &self.favorites_view.get_widget(),
            &self.favorites_view.name,
            &self.favorites_view.title,
        );
        self.widget
            .set_child_icon_name(&self.favorites_view.get_widget(), Some(&self.favorites_view.icon));
        // Archive View
        self.widget
            .add_titled(&self.archive_view.get_widget(), &self.archive_view.name, &self.archive_view.title);
        self.widget
            .set_child_icon_name(&self.archive_view.get_widget(), Some(&self.archive_view.icon));

        self.widget.show();
    }

    pub fn add(&self, article: &Article) {
        if !article.is_starred && !article.is_archived {
            self.unread_view.add(article);
        } else {
            if article.is_starred {
                self.favorites_view.add(article);
            }
            if article.is_archived {
                self.archive_view.add(article);
            }
        }
    }

    pub fn update(&self, article: Article) {}

    pub fn delete(&self, article: &Article) {
        self.remove_from_view(article);
    }

    pub fn favorite(&self, article: &Article) {
        self.remove_from_view(article);
        self.add(article);
    }

    pub fn archive(&self, article: &Article) {
        self.remove_from_view(article);
        self.add(article);
    }

    fn remove_from_view(&self, article: &Article) {
        self.unread_view.delete(article);
        self.archive_view.delete(article);
        self.favorites_view.delete(article);
    }
}
