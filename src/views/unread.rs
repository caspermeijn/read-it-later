use glib::object::Cast;
use glib::Sender;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use crate::application::Action;
use crate::models::{Article, ArticlesModel, ObjectWrapper};
use crate::widgets::articles::{ArticleRow, ArticlesListWidget};

pub struct UnreadView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    model: ArticlesModel,
    sender: Sender<Action>,
}

impl UnreadView {
    pub fn new(sender: Sender<Action>) -> Self {
        let unread_filter = EntriesFilter {
            archive: Some(false),
            starred: Some(false),
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: 0,
            public: None,
        };

        let widget = ArticlesListWidget::new(sender.clone());
        let model = ArticlesModel::new(unread_filter);

        let unread_view = Self {
            widget,
            sender,
            model,
            name: "unread".to_string(),
            title: "Unread".to_string(),
            icon: "unread-symbolic".to_string(),
        };
        unread_view.init();
        unread_view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    fn init(&self) {
        let sender = self.sender.clone();
        self.widget.bind_model(&self.model.model, move |article| {
            let article: Article = article.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
            let row = ArticleRow::new(article.clone(), sender.clone());
            let sender = sender.clone();
            row.set_on_click_callback(move |_, _| {
                sender.send(Action::LoadArticle(article.clone())).unwrap();
                gtk::Inhibit(false)
            });
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
