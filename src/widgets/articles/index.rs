use anyhow::Result;
use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use std::{cell::RefCell, rc::Rc};
use webkit2gtk::traits::{ContextMenuExt, ContextMenuItemExt, WebViewExt};
use webkit2gtk::WebView;

use crate::models::{Article, ArticleAction};
use crate::settings::{Key, SettingsManager};

pub struct ArticleWidget {
    pub widget: gtk::Box,
    builder: gtk::Builder,
    sender: Sender<ArticleAction>,
    pub actions: gio::SimpleActionGroup,
    article: RefCell<Option<Article>>,
}

impl ArticleWidget {
    pub fn new(sender: Sender<ArticleAction>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/article.ui");
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
            for menu_item in context_menu.items() {
                let action = menu_item.stock_action();

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
        webview.connect_estimated_load_progress_notify(move |webview| {
            let progress = webview.estimated_load_progress();
            revealer.set_reveal_child(true);
            progressbar.set_fraction(progress);
            if (progress - 1.0).abs() < std::f64::EPSILON {
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
                    glib::idle_add(clone!(@strong article => move || {
                        let article_url = article.url.clone();
                        let screen = gdk::Screen::default().unwrap();
                        if let Err(err_msg) = gtk::show_uri(Some(&screen), &article_url.unwrap(), 0) {
                            error!("Failed to open the uri {} in the default browser", err_msg);
                        }
                        glib::Continue(false)
                    }));
                }
            })
        );
        // Archive article
        stateful_action!(
            self.actions,
            "archive",
            false,
            clone!(@strong aw, @strong self.sender as sender => move |action, _|{
                let state = action.state().unwrap();
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
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_starred = !action_state;
                action.set_state(&is_starred.to_variant());

                if let Some(article) = aw.article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Favorite(article));
                }
            })
        );
    }

    pub fn load_article(&self, article: Article) -> Result<()> {
        get_widget!(self.builder, WebView, webview);
        info!("Loading the article {:#?}", article.title);
        self.article.replace(Some(article.clone()));

        let mut layout_html = load_resource("layout.html")?;

        if let Some(title) = &article.title {
            layout_html = layout_html.replace("{title}", title);
        }

        if let Some(article_info) = article.get_article_info(true) {
            layout_html = layout_html.replace("{article_info}", &article_info);
        }

        if let Some(content) = &article.content {
            layout_html = layout_html.replace("{content}", content);
        }

        let mut layout_css = load_resource("layout.css")?;
        if SettingsManager::boolean(Key::DarkMode) {
            layout_css.push_str(&load_resource("layout-dark.css")?);
        }
        layout_html = layout_html.replace("{css}", &layout_css);

        let layout_js = load_resource("layout.js")?;
        layout_html = layout_html.replace("{js}", &layout_js);
        webview.load_html(&layout_html, None);

        Ok(())
    }
}

pub fn load_resource(file: &str) -> Result<String> {
    let file = gio::File::for_uri(&format!("resource:///com/belmoussaoui/ReadItLater/{}", file));
    let (bytes, _) = file.load_bytes(gio::NONE_CANCELLABLE)?;
    String::from_utf8(bytes.to_vec()).map_err(From::from)
}
