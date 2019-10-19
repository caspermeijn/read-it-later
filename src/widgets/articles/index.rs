use glib::Sender;
use gtk::prelude::*;
use webkit2gtk::UserContentManager;
use webkit2gtk::WebViewExtManual;
use webkit2gtk::{WebContext, WebView, WebViewExt};

use crate::application::Action;
use crate::models::Article;

pub struct ArticleWidget {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    sender: Sender<Action>,
    webview: WebView,
}

impl ArticleWidget {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article.ui");
        let widget: gtk::Box = builder.get_object("article").expect("Failed to retrieve article");

        let context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());
        let article_widget = Self {
            widget,
            builder,
            sender,
            webview,
        };
        article_widget.init();
        article_widget
    }

    fn init(&self) {
        let article_container: gtk::Box = self.builder.get_object("article_container").expect("Failed to retrieve article_container");

        self.webview.set_property_height_request(130);
        article_container.pack_start(&self.webview, true, true, 0);
        self.webview.show();
    }

    pub fn load_article(&self, article: Article) {
        let title_label: gtk::Label = self.builder.get_object("title_label").expect("Failed to retrieve title_label");

        if let Some(title) = &article.title {
            title_label.set_text(&title);
        }

        let info_label: gtk::Label = self.builder.get_object("info_label").expect("Failed to retrieve info_label");
        if let Some(content) = &article.content {
            self.webview.load_html(&content, None);
        }
    }
}
