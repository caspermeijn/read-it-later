use glib::object::Cast;
use glib::Sender;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use crate::application::Action;
use crate::models::{Article, ArticlesModel};
use crate::widgets::articles::ArticlesListWidget;

pub struct ArchiveView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    sender: Sender<Action>,
    model: ArticlesModel,
}

impl ArchiveView {
    pub fn new(sender: Sender<Action>) -> Self {
        let archive_filter = EntriesFilter {
            archive: Some(true),
            starred: None,
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: 0,
            public: None,
        };
        let widget = ArticlesListWidget::new(sender.clone());
        let model = ArticlesModel::new(archive_filter);

        let archive_view = Self {
            widget,
            sender,
            model,
            name: "archive".to_string(),
            title: "Archive".to_string(),
            icon: "archive-symbolic".to_string(),
        };
        archive_view.init();
        archive_view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    pub fn add(&self, article: Article) {
        self.model.add_article(&article);
    }

    fn init(&self) {
        self.widget.bind_model(&self.model.model);
    }
}
