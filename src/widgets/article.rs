// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;

use anyhow::Result;
use async_std::channel::Sender;
use glib::clone;
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use log::{error, info};
use webkit::{prelude::*, NetworkSession, Settings, WebView};

use crate::models::{Article, ArticleAction};

mod imp {
    use std::cell::OnceCell;

    use glib::subclass::InitializingObject;

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article.ui")]
    pub struct ArticleWidget {
        #[template_child]
        pub webview: TemplateChild<WebView>,
        #[template_child]
        pub revealer: TemplateChild<gtk::Revealer>,
        #[template_child]
        pub progressbar: TemplateChild<gtk::ProgressBar>,
        pub sender: OnceCell<Sender<ArticleAction>>,
        pub actions: gio::SimpleActionGroup,
        pub article: RefCell<Option<Article>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticleWidget {
        const NAME: &'static str = "ArticleWidget";
        type Type = super::ArticleWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            use crate::widgets::articles::ArticlePreview;
            Settings::ensure_type();
            NetworkSession::ensure_type();
            ArticlePreview::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticleWidget {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticleWidget {}

    #[gtk::template_callbacks]
    impl ArticleWidget {
        #[template_callback]
        fn modify_context_menu(_: &WebView, context_menu: &webkit::ContextMenu) -> bool {
            // Right/Left Click context menu
            let forbidden_actions = [
                webkit::ContextMenuAction::OpenLink,
                webkit::ContextMenuAction::OpenLinkInNewWindow,
                webkit::ContextMenuAction::OpenImageInNewWindow,
                webkit::ContextMenuAction::GoBack,
                webkit::ContextMenuAction::GoForward,
                webkit::ContextMenuAction::Stop,
                webkit::ContextMenuAction::Reload,
                webkit::ContextMenuAction::InspectElement,
                webkit::ContextMenuAction::DownloadLinkToDisk,
                webkit::ContextMenuAction::DownloadImageToDisk,
            ];

            for menu_item in context_menu.items() {
                let action = menu_item.stock_action();

                if forbidden_actions.contains(&action) {
                    // Remove forbidden actions
                    context_menu.remove(&menu_item);
                }
            }
            false
        }

        #[template_callback]
        fn decide_policy(
            _: &webkit::WebView,
            decision: &webkit::PolicyDecision,
            decision_type: webkit::PolicyDecisionType,
        ) -> bool {
            if decision_type == webkit::PolicyDecisionType::NavigationAction {
                if let Ok(navigation_decision) = decision
                    .clone()
                    .downcast::<webkit::NavigationPolicyDecision>()
                {
                    if let Some(mut navigation_action) = navigation_decision.navigation_action() {
                        if let webkit::NavigationType::LinkClicked =
                            navigation_action.navigation_type()
                        {
                            if let Some(request) = navigation_action.request() {
                                if let Some(uri) = request.uri() {
                                    // User clicked a link; cancel navigation and open in browser
                                    decision.ignore();
                                    open_uri_in_browser(uri);
                                }
                            }
                        }
                    }
                }
            }
            true
        }

        #[template_callback]
        fn update_load_progress(&self, _pspec: &glib::ParamSpec, webview: &WebView) {
            let progress = webview.estimated_load_progress();
            self.revealer.set_reveal_child(true);
            self.progressbar.set_fraction(progress);
            if (progress - 1.0).abs() < f64::EPSILON {
                self.revealer.set_reveal_child(false);
            }
        }
    }
}

glib::wrapper! {
    pub struct ArticleWidget(ObjectSubclass<imp::ArticleWidget>)
        @extends gtk::Widget;
}

impl ArticleWidget {
    pub fn set_sender(&self, sender: Sender<ArticleAction>) {
        self.imp().sender.set(sender).unwrap();
        self.setup_actions();
    }

    fn setup_actions(&self) {
        self.imp().actions.add_action_entries([
            // Delete article
            gio::ActionEntry::builder("delete")
                .activate(clone!(@strong self as aw => move|_, _, _|{
                  let imp = aw.imp();
                  let sender = imp.sender.get().unwrap();
                  if let Some(article) = imp.article.borrow().clone(){
                    sender.send_blocking(ArticleAction::Delete(article)).unwrap();
                  }
                }))
                .build(),
            // Share article
            gio::ActionEntry::builder("open")
                .activate(clone!(@strong self as aw => move|_, _, _|{
                  if let Some(article) = aw.imp().article.borrow().clone(){
                    let article_url = article.url.clone().unwrap();
                    open_uri_in_browser(article_url);
                  }
                }))
                .build(),
            // Archive article
            gio::ActionEntry::builder("archive")
                .state(false.into())
                .activate(clone!(@strong self as aw => move |_, action, _|{
                    let imp = aw.imp();
                    let sender = imp.sender.get().unwrap();
                    let state = action.state().unwrap();
                    let action_state: bool = state.get().unwrap();
                    let is_archived = !action_state;
                    action.set_state(&is_archived.into());
                    if let Some(article) = imp.article.borrow_mut().clone() {
                        sender.send_blocking(ArticleAction::Archive(article)).unwrap();
                    }
                }))
                .build(),
            // Favorite article
            gio::ActionEntry::builder("favorite")
                .state(false.into())
                .activate(clone!(@strong self as aw => move |_, action, _|{
                    let imp = aw.imp();
                    let sender = imp.sender.get().unwrap();
                    let state = action.state().unwrap();
                    let action_state: bool = state.get().unwrap();
                    let is_starred = !action_state;
                    action.set_state(&is_starred.into());

                    if let Some(article) = imp.article.borrow_mut().clone() {
                        sender.send_blocking(ArticleAction::Favorite(article)).unwrap();
                    }
                }))
                .build(),
        ]);
    }

    pub fn load(&self, article: Article) {
        if let Err(err) = self.try_load(article) {
            error!("Failed to load article {}", err);
        }
    }

    fn try_load(&self, article: Article) -> Result<()> {
        info!("Loading the article {:#?}", article.title);
        self.imp().article.replace(Some(article.clone()));

        let mut layout_html = load_resource("layout.html")?;

        if let Some(title) = &article.title {
            layout_html = layout_html.replace("{title}", title);
        }

        let article_info = article.get_article_info(true);
        layout_html = layout_html.replace("{article_info}", &article_info);

        if let Some(content) = &article.content {
            layout_html = layout_html.replace("{content}", content);
        }

        let layout_css = load_resource("layout.css")?;
        layout_html = layout_html.replace("{css}", &layout_css);

        let layout_js = load_resource("layout.js")?;
        layout_html = layout_html.replace("{js}", &layout_js);
        self.imp().webview.load_html(&layout_html, None);

        Ok(())
    }

    pub fn get_actions(&self) -> &gio::SimpleActionGroup {
        &self.imp().actions
    }

    pub fn get_action(&self, name: &str) -> gio::SimpleAction {
        self.get_actions()
            .lookup_action(name)
            .unwrap_or_else(|| panic!("Could not find action \"{}\"", name))
            .downcast::<gio::SimpleAction>()
            .unwrap()
    }

    pub fn set_enable_actions(&self, state: bool) {
        self.get_action("open").set_enabled(state);
        self.get_action("archive").set_enabled(state);
        self.get_action("delete").set_enabled(state);
        self.get_action("favorite").set_enabled(state);
    }
}

pub fn load_resource(file: &str) -> Result<String> {
    let file = gio::File::for_uri(&format!(
        "resource:///com/belmoussaoui/ReadItLater/{}",
        file
    ));
    let (bytes, _) = file.load_bytes(gio::Cancellable::NONE)?;
    String::from_utf8(bytes.to_vec()).map_err(From::from)
}

fn open_uri_in_browser(uri: impl Into<glib::GString>) {
    gtk::UriLauncher::builder().uri(uri).build().launch(
        gtk::Window::NONE,
        gio::Cancellable::NONE,
        |result| {
            if let Err(error) = result {
                log::error!("Failed to launch URI: {}", error)
            }
        },
    );
}
