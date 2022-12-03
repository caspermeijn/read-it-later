use gio::prelude::*;
use gtk::{
    gio,
    glib::{object::Cast, Sender},
};
use gtk_macros::get_action;
use log::error;

use crate::{
    models::{Article, ArticleAction},
    widgets::articles::ArticleWidget,
};

#[derive(Clone)]
pub struct ArticleView {
    widget: ArticleWidget,
    pub name: String,
}

impl ArticleView {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let widget = ArticleWidget::new(sender);

        let article_view = Self {
            widget,
            name: "article".to_string(),
        };
        article_view.init();
        article_view
    }

    pub fn get_actions(&self) -> &gio::SimpleActionGroup {
        self.widget.get_actions()
    }

    pub fn set_enable_actions(&self, state: bool) {
        let action_group = self.get_actions();
        get_action!(action_group, @open).set_enabled(state);
        get_action!(action_group, @archive).set_enabled(state);
        get_action!(action_group, @delete).set_enabled(state);
        get_action!(action_group, @favorite).set_enabled(state);
    }

    pub fn get_widget(&self) -> &ArticleWidget {
        &self.widget
    }

    pub fn load(&self, article: Article) {
        if let Err(err) = self.widget.load_article(article) {
            error!("Failed to load article {}", err);
        }
    }

    fn init(&self) {}
}
