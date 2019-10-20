use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use std::{cell::RefCell, rc::Rc};
use webkit2gtk::UserContentManager;
use webkit2gtk::WebViewExtManual;
use webkit2gtk::{SettingsExt, WebContext, WebView, WebViewExt};

use crate::application::Action;
use crate::models::Article;

pub struct ArticleWidget {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    sender: Sender<Action>,
    webview: WebView,
    article: Rc<RefCell<Option<Article>>>,
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
            article: Rc::new(RefCell::new(None)),
        };
        article_widget.init();
        article_widget.setup_actions();
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

    fn setup_actions(&self) {
        let actions = gio::SimpleActionGroup::new();

        // Delete article
        let delete_article = gio::SimpleAction::new("delete", None);
        let weak_article = Rc::downgrade(&self.article);
        delete_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        actions.add_action(&delete_article);
        // Search article
        let search_article = gio::SimpleAction::new("search", None);
        let weak_article = Rc::downgrade(&self.article);
        search_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        actions.add_action(&search_article);
        // Share article
        let share_article = gio::SimpleAction::new("share", None);
        let weak_article = Rc::downgrade(&self.article);
        share_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        actions.add_action(&share_article);
        // Archive article
        let archive_article = gio::SimpleAction::new("archive", None);
        let weak_article = Rc::downgrade(&self.article);
        archive_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        actions.add_action(&archive_article);
        // Favorite article
        let favorite_article = gio::SimpleAction::new("favorite", None);
        let weak_article = Rc::downgrade(&self.article);
        favorite_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        actions.add_action(&favorite_article);
        self.widget.insert_action_group("article", Some(&actions));
    }

    pub fn load_article(&self, article: Article) {
        self.article.replace(Some(article.clone()));

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
