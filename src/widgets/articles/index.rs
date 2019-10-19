use crate::application::Action;
use crate::models::{Article, PreviewImageType};
use gio::prelude::*;
use gio::FileExt;
use glib::Sender;
use gtk::prelude::*;
use webkit2gtk::UserContentManager;
use webkit2gtk::WebViewExtManual;
use webkit2gtk::{SettingsExt, WebContext, WebView, WebViewExt};

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
        let webview_settings = webkit2gtk::Settings::new();
        webview_settings.set_auto_load_images(true);
        webview_settings.set_enable_developer_extras(true);
        self.webview.set_settings(&webview_settings);

        self.widget.pack_start(&self.webview, true, true, 0);
        self.webview.show();
    }

    pub fn load_article(&self, article: Article) {
        let layout_html = gio::File::new_for_uri("resource:///com/belmoussaoui/ReadItLater/layout.html.in");

        if let Ok((v, _)) = layout_html.load_bytes(gio::NONE_CANCELLABLE) {
            let mut article_content = String::from_utf8(v.to_vec()).unwrap();
            if let Some(title) = &article.title {
                article_content = article_content.replace("{title}", title);
            }

            article_content = article_content.replace("{article_info}", &article.get_info());

            let layout_css = gio::File::new_for_uri("resource:///com/belmoussaoui/ReadItLater/layout.css.in");
            if let Ok((v, _)) = layout_css.load_bytes(gio::NONE_CANCELLABLE) {
                let css_content = String::from_utf8(v.to_vec()).unwrap();
                article_content = article_content.replace("{css}", &css_content);
            }

            if let Some(content) = &article.content {
                article_content = article_content.replace("{content}", content);
            }

            // Some(&article.base_url.unwrap())
            self.webview.load_html(&article_content, None);
        }
    }
}
