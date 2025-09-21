// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2020 Florian Müllner <fmuellner@gnome.org>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::{prelude::*, subclass::prelude::*};
use async_std::channel::Sender;
use glib::{clone, Object};
use gtk::{gio, glib};

use crate::{
    application::Action,
    config::PROFILE,
    models::{Article, ArticlesManager},
    views::{ArticlesView, Login},
    widgets::ArticleWidget,
};

mod imp {
    use std::cell::OnceCell;

    use glib::subclass::InitializingObject;

    use super::*;
    use crate::widgets::new_article::NewArticle;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/window.ui")]
    pub struct Window {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub main_stack: TemplateChild<adw::NavigationView>,

        #[template_child]
        pub login_view: TemplateChild<Login>,
        #[template_child]
        pub article_widget: TemplateChild<ArticleWidget>,
        #[template_child]
        pub articles_view: TemplateChild<ArticlesView>,

        pub sender: OnceCell<Sender<Action>>,
        pub actions: gio::SimpleActionGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();

            klass.install_action("win.new-article", None, move |window, _, _| {
                let sender = window.imp().sender.get().unwrap().clone();
                let dialog = NewArticle::new(sender);
                dialog.present(Some(window));
            });
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for Window {}

    impl WindowImpl for Window {}

    impl ApplicationWindowImpl for Window {}

    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
    @implements gio::ActionGroup, gio::ActionMap, gtk::ConstraintTarget, gtk::Buildable, gtk::Accessible, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum View {
    Article,       // Article
    Login,         // Sign in
    Articles,      // Unread articles
    Syncing(bool), // During sync
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let window: Self = Object::new();
        window.init(sender);
        window
    }

    pub fn load_article(&self, article: Article) {
        let article_widget = self.imp().article_widget.get();
        article_widget
            .get_action("archive")
            .set_state(&article.is_archived.into());
        article_widget
            .get_action("favorite")
            .set_state(&article.is_starred.into());
        article_widget.load(article);
        self.set_view(View::Article);
    }

    pub fn add_toast(&self, toast: adw::Toast) {
        self.imp().toast_overlay.add_toast(toast);
    }

    fn navigation_stack_has_page_tag(&self, tag: &str) -> bool {
        self.imp()
            .main_stack
            .navigation_stack()
            .iter::<adw::NavigationPage>()
            .any(|page| page.unwrap().tag().unwrap() == tag)
    }

    pub fn set_view(&self, view: View) {
        let imp = self.imp();
        self.set_default_widget(gtk::Widget::NONE);
        match view {
            View::Article => {
                imp.main_stack.push_by_tag("article");
            }
            View::Articles => {
                if self.navigation_stack_has_page_tag("articles") {
                    imp.main_stack.pop_to_tag("articles");
                } else {
                    imp.main_stack.replace_with_tags(&["articles"]);
                }
            }
            View::Login => {
                imp.main_stack.replace_with_tags(&["login"]);

                self.set_default_widget(Some(imp.login_view.get_login_button()));
            }
            View::Syncing(state) => {
                imp.articles_view.set_progress_bar_pulsing(state);
                imp.article_widget.set_progress_bar_pulsing(state);
            }
        }
    }

    pub fn init(&self, sender: Sender<Action>) {
        let imp = self.imp();

        if PROFILE == "Devel" {
            self.add_css_class("devel");
        }
        let articles_manager = ArticlesManager::new(sender.clone());

        imp.sender.set(sender).unwrap();

        imp.article_widget
            .set_sender(articles_manager.sender.clone());

        imp.articles_view
            .set_sender(articles_manager.sender.clone());

        self.init_views();
    }

    fn init_views(&self) {
        let imp = self.imp();

        // Article View
        let article_widget = imp.article_widget.get();
        self.insert_action_group("article", Some(article_widget.get_actions()));

        imp.main_stack.connect_visible_page_notify(clone!(
            #[strong]
            article_widget,
            move |stack| {
                if let Some(page) = stack.visible_page() {
                    if let Some(view_name) = page.tag() {
                        article_widget.set_enable_actions(view_name == "article");
                    }
                }
            }
        ));

        self.set_view(View::Login);
    }

    pub fn articles_view(&self) -> &ArticlesView {
        &self.imp().articles_view
    }
}
