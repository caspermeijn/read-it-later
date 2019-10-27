use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use std::{cell::RefCell, rc::Rc};
use webkit2gtk::UserContentManager;
use webkit2gtk::WebViewExtManual;
use webkit2gtk::{SettingsExt, WebContext, WebView, WebViewExt};

use crate::application::Action;
use crate::models::Article;
use crate::settings::{Key, SettingsManager};

pub struct ArticleWidget {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    sender: Sender<Action>,
    pub actions: gio::SimpleActionGroup,
    webview: WebView,
    article: Rc<RefCell<Option<Article>>>,
}

impl ArticleWidget {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article.ui");
        let widget: gtk::Box = builder.get_object("article").expect("Failed to retrieve article");

        let context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());

        let actions = gio::SimpleActionGroup::new();

        let article_widget = Self {
            widget,
            builder,
            actions,
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
        // Delete article
        let delete_article = gio::SimpleAction::new("delete", None);
        let weak_article = Rc::downgrade(&self.article);
        delete_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        self.actions.add_action(&delete_article);
        // Search article
        let search_article = gio::SimpleAction::new("search", None);
        let weak_article = Rc::downgrade(&self.article);
        search_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        self.actions.add_action(&search_article);
        // Share article
        let open_article = gio::SimpleAction::new("open", None);
        let weak_article = Rc::downgrade(&self.article);
        open_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                let article_url = article.borrow_mut().take().unwrap().url;
                gtk::show_uri(Some(&gdk::Screen::get_default().unwrap()), &article_url.unwrap(), 0);
                // icon.copy_name();
            }
        });
        self.actions.add_action(&open_article);
        // Archive article
        let archive_article = gio::SimpleAction::new("archive", None);
        let weak_article = Rc::downgrade(&self.article);
        archive_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        self.actions.add_action(&archive_article);
        // Favorite article
        let favorite_article = gio::SimpleAction::new("favorite", None);
        let weak_article = Rc::downgrade(&self.article);
        favorite_article.connect_activate(move |_, _| {
            if let Some(article) = weak_article.upgrade() {
                // icon.copy_name();
            }
        });
        self.actions.add_action(&favorite_article);
    }

    pub fn load_article(&self, article: Article) {
        self.article.replace(Some(article.clone()));

        let layout_html = gio::File::new_for_uri("resource:///com/belmoussaoui/ReadItLater/layout.html");

        if let Ok((v, _)) = layout_html.load_bytes(gio::NONE_CANCELLABLE) {
            let mut article_content = String::from_utf8(v.to_vec()).unwrap();
            if let Some(title) = &article.title {
                article_content = article_content.replace("{title}", title);
            }

            let mut article_info = String::from("");
            if let Some(base_url) = &article.base_url {
                article_info.push_str(&format!("{} | ", base_url));
            }
            if let Some(authors) = &article.published_by {
                article_info.push_str(&format!("by {} ", authors));
            }
            if let Some(published_date) = &article.published_at {
                article_info.push_str(&format!("on {} ", published_date));
            }
            article_content = article_content.replace("{article_info}", &article_info);

            let layout_css = gio::File::new_for_uri("resource:///com/belmoussaoui/ReadItLater/layout.css");
            if let Ok((v, _)) = layout_css.load_bytes(gio::NONE_CANCELLABLE) {
                let mut css_content = String::from_utf8(v.to_vec()).unwrap();

                if SettingsManager::get_boolean(Key::DarkMode) {
                    let layout_css = gio::File::new_for_uri("resource:///com/belmoussaoui/ReadItLater/layout-dark.css");
                    if let Ok((v, _)) = layout_css.load_bytes(gio::NONE_CANCELLABLE) {
                        css_content.push_str(&String::from_utf8(v.to_vec()).unwrap());
                    }
                }

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
