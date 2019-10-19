use glib::object::Cast;
use glib::Sender;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use crate::application::Action;
use crate::widgets::articles::ArticlesListWidget;

pub struct ArchiveView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    sender: Sender<Action>,
}

impl ArchiveView {
    pub fn new(sender: Sender<Action>) -> Self {
        let widget = ArticlesListWidget::new(sender.clone());

        let archive_view = Self {
            widget,
            sender,
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

    fn init(&self) {}
}
