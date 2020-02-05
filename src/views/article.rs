use gio::prelude::*;
use glib::object::Cast;
use glib::Sender;

use crate::models::{Article, ArticleAction};
use crate::widgets::articles::ArticleWidget;

#[derive(Clone)]
pub struct ArticleView {
    widget: std::rc::Rc<ArticleWidget>,
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

    pub fn get_actions(&self) -> Option<&gio::SimpleActionGroup> {
        Some(&self.widget.actions)
    }

    pub fn set_enable_actions(&self, state: bool) {
        let actions = &self.widget.actions;
        actions.list_actions().iter().for_each(|action_name| {
            let action = actions
                .lookup_action(action_name.as_str())
                .unwrap()
                .downcast::<gio::SimpleAction>()
                .unwrap();
            action.set_enabled(state);
        });
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    pub fn load(&self, article: Article) {
        if let Err(err) = self.widget.load_article(article) {
            error!("Failed to load article {}", err);
        }
    }

    fn init(&self) {}
}
