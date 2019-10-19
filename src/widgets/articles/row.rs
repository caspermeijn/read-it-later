use crate::application::Action;
use crate::models::Article;
use glib::Sender;
use gtk::prelude::*;
use webkit2gtk::UserContentManager;
use webkit2gtk::WebViewExtManual;
use webkit2gtk::{SettingsExt, WebContext, WebContextExt, WebView, WebViewExt};

pub struct ArticleRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    sender: Sender<Action>,
    article: Article,
}

impl ArticleRow {
    pub fn new(article: Article, sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article_row.ui");
        let widget: gtk::ListBoxRow = builder.get_object("article_row").expect("Failed to retrieve article_row");

        let row = Self {
            widget,
            builder,
            sender,
            article,
        };

        row.init();
        row
    }

    pub fn set_on_click_callback<F>(&self, callback: F)
    where
        for<'r, 's> F: std::ops::Fn(&'r gtk::EventBox, &'s gdk::EventButton) -> gtk::Inhibit + 'static,
    {
        let event_box: gtk::EventBox = self.builder.get_object("event_box").expect("Failed to load event_box");
        event_box.connect_button_press_event(callback);
    }

    fn init(&self) {
        let content_box: gtk::Box = self.builder.get_object("content_box").expect("failed to retrieve content_box");
        let title_label: gtk::Label = self.builder.get_object("title_label").expect("Failed to retrieve title_label");

        if let Some(title) = &self.article.title {
            title_label.set_text(&title);
        }

        let info_label: gtk::Label = self.builder.get_object("info_label").expect("Failed to retrieve info_label");
        let context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());
        if let Some(content) = &self.article.content {
            webview.load_html(&content, None);
        }
        webview.set_property_height_request(130);
        content_box.add(&webview);
        webview.show();

        let preview_image: gtk::Image = self.builder.get_object("preview_image").expect("Failed to retrieve preview_image");
        if let Some(pixbuf) = &self.article.get_preview_pixbuf() {
            //            preview_image.
        } else {
            preview_image.set_no_show_all(false);
            preview_image.hide();
        }
    }
}
