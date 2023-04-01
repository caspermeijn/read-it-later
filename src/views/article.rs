use gtk::{gio, glib::Sender};

use crate::{
    models::{Article, ArticleAction},
    widgets::articles::ArticleWidget,
};

#[derive(Clone, Debug)]
pub struct ArticleView {
    widget: ArticleWidget,
}

impl ArticleView {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let widget = ArticleWidget::new();
        widget.set_sender(sender);

        Self { widget }
    }

    pub fn get_actions(&self) -> &gio::SimpleActionGroup {
        self.widget.get_actions()
    }

    pub fn set_enable_actions(&self, state: bool) {
        self.widget.set_enable_actions(state)
    }

    pub fn get_widget(&self) -> &ArticleWidget {
        &self.widget
    }

    pub fn load(&self, article: Article) {
        self.widget.load(article)
    }
}
