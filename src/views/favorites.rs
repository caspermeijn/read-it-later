use glib::object::Cast;
use glib::Sender;
use wallabag_api::types::{EntriesFilter, SortBy, SortOrder};

use crate::application::Action;
use crate::models::{Article, ArticlesModel, ObjectWrapper};
use crate::widgets::articles::{ArticleRow, ArticlesListWidget};

pub struct FavoritesView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    sender: Sender<Action>,
    model: ArticlesModel,
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
        let model = ArticlesModel::new(favorites_filter);

        let favorites_view = Self {
            widget,
            sender,
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
