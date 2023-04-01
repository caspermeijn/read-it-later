use glib::Sender;
use gtk::{gio, gio::prelude::*, glib};

use crate::{
    models::{ArticleAction, ArticlesFilter},
    widgets::articles::ArticlesListWidget,
};

#[derive(Clone, Debug)]
pub struct ArticlesListView {
    widget: ArticlesListWidget,
    pub name: String,
    pub title: String,
    pub icon: String,
    model: gtk::FilterListModel,
}

impl ArticlesListView {
    pub fn new(
        name: &str,
        title: &str,
        icon: &str,
        filter: ArticlesFilter,
        sender: Sender<ArticleAction>,
        model: gio::ListStore,
    ) -> Self {
        let filter: gtk::Filter = filter.into();
        let model = gtk::FilterListModel::new(Some(model), Some(filter));
        let widget = ArticlesListWidget::new(sender);

        let articles_view = Self {
            widget,
            model,
            name: name.to_string(),
            title: title.to_string(),
            icon: icon.to_string(),
        };
        articles_view.init();
        articles_view
    }

    pub fn get_widget(&self) -> &ArticlesListWidget {
        &self.widget
    }

    fn init(&self) {
        self.widget
            .set_property("placeholder-icon-name", &self.icon);
        self.widget.bind_model(&self.model);
    }
}
