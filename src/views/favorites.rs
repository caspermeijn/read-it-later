use glib::object::Cast;
use glib::Sender;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use crate::application::Action;
use crate::models::{Article, ArticlesModel};
use crate::widgets::articles::ArticlesListWidget;

pub struct FavoritesView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    model: ArticlesModel,
}

impl FavoritesView {
    pub fn new(sender: Sender<Action>) -> Self {
        let unread_filter = EntriesFilter {
            archive: None,
            starred: Some(true),
            sort: SortBy::Created,
            order: SortOrder::Desc,
            tags: vec![],
            since: 0,
            public: None,
        };

        let widget = ArticlesListWidget::new(sender.clone());
        let model = ArticlesModel::new(unread_filter);

        let favorites_view = Self {
            widget,
            model,
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

    pub fn add(&self, article: Article) {
        self.model.add_article(&article);
    }

    pub fn delete(&self, article: Article) {
        self.model.remove_article(&article);
    }
    fn init(&self) {
        self.widget
            .bind_model(&self.model.model, &self.icon, "Save your favorites articles!");
    }
}
