use glib::object::Cast;
use glib::Sender;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use crate::application::Action;
use crate::widgets::articles::ArticlesListWidget;

pub struct FavoritesView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    sender: Sender<Action>,
}

impl FavoritesView {
    pub fn new(sender: Sender<Action>) -> Self {
        let favorites_filter = EntriesFilter {
            archive: Some(false),
            starred: Some(true),
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: 0,
            public: None,
        };

        let widget = ArticlesListWidget::new(sender.clone());

        let favorites_view = Self {
            widget,
            sender,
            name: "favorites".to_string(),
            title: "Favorites".to_string(),
            icon: "favorites-symbolic".to_string(),
        };
        favorites_view.init();
        favorites_view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    fn init(&self) {}
}
