use glib::object::Cast;
use glib::Sender;

use crate::models::Article;
use crate::widgets::articles::ArticleWidget;

use crate::application::Action;

pub struct ArticleView {
    widget: ArticleWidget,
    pub name: String,
    sender: Sender<Action>,
}

impl ArticleView {
    pub fn new(sender: Sender<Action>) -> Self {
        let widget = ArticleWidget::new(sender.clone());

        let article_view = Self {
            widget,
            sender,
            name: "article".to_string(),
        };
        article_view.init();
        article_view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    pub fn load(&self, article: Article) {
        self.widget.load_article(article);
    }

    fn init(&self) {}
}
