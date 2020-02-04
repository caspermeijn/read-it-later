use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use std::{cell::RefCell, rc::Rc};
use webkit2gtk::{ContextMenuExt, ContextMenuItemExt, WebView, WebViewExt};

use crate::models::{Article, ArticleAction};
use crate::settings::{Key, SettingsManager};
use crate::utils;

pub struct ArticleWidget {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    sender: Sender<ArticleAction>,
    pub actions: gio::SimpleActionGroup,
    article: RefCell<Option<Article>>,
}

impl ArticleWidget {
    pub fn new(sender: Sender<ArticleAction>) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article.ui");
        get_widget!(builder, gtk::Box, article);

        let actions = gio::SimpleActionGroup::new();

        let article_widget = Rc::new(Self {
            widget: article,
            builder,
            actions,
            sender,
            article: RefCell::new(None),
        });
        article_widget.init();
        article_widget.setup_actions(article_widget.clone());
        article_widget
    }

    fn init(&self) {
        // Right/Left Click context menu
        let forbidden_actions = vec![
            webkit2gtk::ContextMenuAction::OpenLink,
            webkit2gtk::ContextMenuAction::GoBack,
            webkit2gtk::ContextMenuAction::GoForward,
            webkit2gtk::ContextMenuAction::Stop,
            webkit2gtk::ContextMenuAction::Reload,
            webkit2gtk::ContextMenuAction::InspectElement,
        ];
        get_widget!(self.builder, WebView, webview);

        webview.connect_context_menu(move |_, context_menu, _, _| {
            for menu_item in context_menu.get_items() {
                let action = menu_item.get_stock_action();

                if forbidden_actions.contains(&action) {
                    // Remove forbidden actions
                    context_menu.remove(&menu_item);
                }
            }
            false
        });

        // Progress bar
        get_widget!(self.builder, gtk::Revealer, revealer);
        get_widget!(self.builder, gtk::ProgressBar, progressbar);
        webview.connect_property_estimated_load_progress_notify(move |webview| {
            let progress = webview.get_estimated_load_progress();

            progressbar.set_fraction(progress);
            if progress == 1.0 {
                revealer.set_reveal_child(false);
            }
        });
    }

    fn setup_actions(&self, aw: Rc<Self>) {
        // Delete article
        action!(
            self.actions,
            "delete",
            clone!(@strong aw, @strong self.sender as sender => move |_, _| {
                if let Some(article) = aw.article.borrow().clone() {
                    send!(sender, ArticleAction::Delete(article));
                }
            })
        );
        // Share article
        action!(
            self.actions,
            "open",
            clone!(@strong aw => move |_, _| {
                if let Some(article) = aw.article.borrow().clone() {
                    let article_url = article.url;
                    let screen = gdk::Screen::get_default().unwrap();
                    if let Err(err_msg) = gtk::show_uri(Some(&screen), &article_url.unwrap(), 0) {
                        error!("Failed to open the uri {} in the default browser", err_msg);
                    }
                }
            })
        );
        // Archive article
        stateful_action!(
            self.actions,
            "archive",
            false,
            clone!(@strong aw, @strong self.sender as sender => move |action, _|{
                let state = action.get_state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_archived = !action_state;
                action.set_state(&is_archived.to_variant());
                if let Some(article) = aw.article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Archive(article));
                }
            })
        );
        // Favorite article
        stateful_action!(
            self.actions,
            "favorite",
            false,
            clone!(@strong aw, @strong self.sender as sender => move |action, _|{
                let state = action.get_state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_starred = !action_state;
                action.set_state(&is_starred.to_variant());

                if let Some(article) = aw.article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Favorite(article));
                }
            })
        );
    }

    pub fn load_article(&self, article: Article) {
        get_widget!(self.builder, WebView, webview);
        info!("Loading the article {:#?}", article.title);
        self.article.replace(Some(article.clone()));
        // Progress Bar Revealer
        get_widget!(self.builder, gtk::Revealer, revealer);
        revealer.set_reveal_child(true);

        if let Ok(mut layout_html) = utils::load_resource("layout.html") {
            if let Some(title) = &article.title {
                layout_html = layout_html.replace("{title}", title);
            }

            if let Some(article_info) = article.get_article_info(true) {
                layout_html = layout_html.replace("{article_info}", &article_info);
            }

            if let Some(content) = &article.content {
                layout_html = layout_html.replace("{content}", content);
            }

            let mut layout_css = utils::load_resource("layout.css").expect("Couldn't find the article layout css");
            if SettingsManager::get_boolean(Key::DarkMode) {
                layout_css.push_str(&utils::load_resource("layout-dark.css").expect("Couldn't find the article dark layout css"));
            }
            layout_html = layout_html.replace("{css}", &layout_css);

            let layout_js = utils::load_resource("layout.js").expect("Couldn't find the article layout js");
            layout_html = layout_html.replace("{js}", &layout_js);
            println!("hey world {}", layout_html);
            webview.load_html(&layout_html, None);
        }
    }
}
