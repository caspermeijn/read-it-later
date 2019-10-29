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
    article: RefCell<Option<Article>>,
}

impl ArticleWidget {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article.ui");
        let widget: gtk::Box = builder.get_object("article").expect("Failed to retrieve article");

        let context = WebContext::get_default().unwrap();
        let webview = WebView::new_with_context_and_user_content_manager(&context, &UserContentManager::new());

        let actions = gio::SimpleActionGroup::new();

        let article_widget = Rc::new(Self {
            widget,
            builder,
            actions,
            sender,
            webview,
            article: RefCell::new(None),
        });
        article_widget.init();
        article_widget.setup_actions(article_widget.clone());
        article_widget
    }

    fn init(&self) {
        let webview_settings = webkit2gtk::Settings::new();
        webview_settings.set_auto_load_images(true);
        // webview_settings.set_enable_javascript(false);
        webview_settings.set_allow_modal_dialogs(false);
        webview_settings.set_enable_developer_extras(false);
        self.webview.set_settings(&webview_settings);

        // Progress bar
        get_widget!(self.builder, gtk::Revealer, revealer);
        get_widget!(self.builder, gtk::ProgressBar, progressbar);

        self.webview.connect_property_estimated_load_progress_notify(move |webview| {
            let progress = webview.get_estimated_load_progress();

            progressbar.set_fraction(progress);
            if progress == 1.0 {
                revealer.set_reveal_child(false);
            }
        });

        self.widget.pack_start(&self.webview, true, true, 0);
        self.webview.show();
    }

    fn setup_actions(&self, aw: Rc<Self>) {
        // Delete article
        let delete_article = gio::SimpleAction::new("delete", None);
        let article_widget = aw.clone();
        let sender = self.sender.clone();
        delete_article.connect_activate(move |_, _| {
            if let Some(article) = article_widget.article.borrow().clone() {
                sender.send(Action::DeleteArticle(article)).expect("Failed to delete the article");
            }
        });
        self.actions.add_action(&delete_article);
        // Search article
        let search_article = gio::SimpleAction::new("search", None);
        let article_widget = aw.clone();
        let sender = self.sender.clone();
        search_article.connect_activate(move |_, _| if let Some(article) = article_widget.article.borrow().clone() {});
        self.actions.add_action(&search_article);
        // Share article
        let open_article = gio::SimpleAction::new("open", None);
        let article_widget = aw.clone();
        open_article.connect_activate(move |_, _| {
            if let Some(article) = article_widget.article.borrow().clone() {
                let article_url = article.url;
                if let Err(err_msg) = gtk::show_uri(Some(&gdk::Screen::get_default().unwrap()), &article_url.unwrap(), 0) {
                    error!("Failed to open the uri {} in the default browser", err_msg);
                }
            }
        });
        self.actions.add_action(&open_article);
        // Archive article
        let is_archived = false; // false by default
        let archive_article = gio::SimpleAction::new_stateful("archive", None, &is_archived.to_variant());
        let article_widget = aw.clone();
        let sender = self.sender.clone();
        archive_article.connect_activate(move |action, _| {
            let state = action.get_state().unwrap();
            let action_state: bool = state.get().unwrap();
            let is_archived = !action_state;
            action.set_state(&is_archived.to_variant());

            if let Some(article) = article_widget.article.borrow().clone() {
                sender.send(Action::ArchiveArticle(article)).expect("Failed to archive the article");
            }
        });
        self.actions.add_action(&archive_article);
        // Favorite article
        let is_starred = false; // false by default
        let favorite_article = gio::SimpleAction::new_stateful("favorite", None, &is_starred.to_variant());
        let article_widget = aw.clone();
        let sender = self.sender.clone();
        favorite_article.connect_activate(move |action, _| {
            let state = action.get_state().unwrap();
            let action_state: bool = state.get().unwrap();
            let is_starred = !action_state;
            action.set_state(&is_starred.to_variant());

            if let Some(article) = article_widget.article.borrow().clone() {
                sender.send(Action::FavoriteArticle(article)).expect("Failed to favorite the article");
            }
        });
        self.actions.add_action(&favorite_article);
    }

    pub fn load_article(&self, article: Article) {
        self.article.replace(Some(article.clone()));
        // Progress Bar Revealer
        get_widget!(self.builder, gtk::Revealer, revealer);
        revealer.set_reveal_child(true);

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
