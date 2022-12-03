use adw::prelude::*;
use glib::{clone, timeout_future_seconds, MainContext, Sender};
use gtk::{gio, glib};
use gtk_macros::{action, get_action, get_widget, send};
use log::error;
use url::Url;

use crate::{
    application::Action,
    config::PROFILE,
    models::{Article, ArticlesManager},
    views::{ArticleView, ArticlesView, Login},
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum View {
    Article,       // Article
    Login,         // Sign in
    Articles,      // Unread articles
    Syncing(bool), // During sync
    NewArticle,    // New Article
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    builder: gtk::Builder,
    sender: Sender<Action>,
    article_view: ArticleView,
    pub articles_view: ArticlesView,
    login_view: Login,
    actions: gio::SimpleActionGroup,
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/window.ui");
        get_widget!(builder, gtk::ApplicationWindow, window);

        if PROFILE == "Devel" {
            window.add_css_class("devel");
        }
        let actions = gio::SimpleActionGroup::new();

        let articles_manager = ArticlesManager::new(sender.clone());

        let window_widget = Window {
            widget: window,
            builder,
            article_view: ArticleView::new(articles_manager.sender.clone()),
            articles_view: ArticlesView::new(articles_manager.sender.clone()),
            login_view: Login::new(),
            sender,
            actions,
        };

        window_widget.init();
        window_widget.init_views();
        window_widget.setup_actions();
        window_widget
    }

    pub fn load_article(&self, article: Article) {
        let article_view_actions = self.article_view.get_actions();
        get_action!(article_view_actions, @archive).set_state(&article.is_archived.to_variant());
        get_action!(article_view_actions, @favorite).set_state(&article.is_starred.to_variant());
        self.article_view.load(article);
        self.set_view(View::Article);
    }

    pub fn notify(&self, message: String) {
        get_widget!(self.builder, gtk::Revealer, notification);
        get_widget!(self.builder, gtk::Label, notification_label);

        notification_label.set_text(&message);
        notification.set_reveal_child(true);

        let main_context = MainContext::default();
        main_context.spawn_local(clone!(@weak notification => async move {
            timeout_future_seconds(5).await;
            notification.set_reveal_child(false);
        }));
    }

    pub fn previous_view(&self) {
        self.set_view(View::Articles);
    }

    pub fn set_view(&self, view: View) {
        get_widget!(self.builder, gtk::ApplicationWindow, window);
        window.set_default_widget(gtk::Widget::NONE);
        get_widget!(self.builder, gtk::Stack, main_stack);
        get_widget!(self.builder, gtk::Stack, headerbar_stack);
        match view {
            View::Article => {
                main_stack.set_visible_child_name("article");
                headerbar_stack.set_visible_child_name("article");
            }
            View::Articles => {
                main_stack.set_visible_child_name("articles");
                headerbar_stack.set_visible_child_name("articles");
            }
            View::Login => {
                main_stack.set_visible_child_name("login");
                headerbar_stack.set_visible_child_name("login");

                window.set_default_widget(Some(self.login_view.get_login_button()));
            }
            View::Syncing(state) => {
                get_widget!(self.builder, gtk::ProgressBar, loading_progress);
                loading_progress.set_visible(state);
                if !state {
                    // If we hide the progress bar
                    loading_progress.set_fraction(0.0); // Reset the fraction
                } else {
                    let main_context = MainContext::default();

                    loading_progress.pulse();

                    let future = clone!(@weak loading_progress => async move {
                        timeout_future_seconds(1).await;
                        loading_progress.pulse();
                    });

                    main_context.spawn_local(future);
                }
            }
            View::NewArticle => {
                headerbar_stack.set_visible_child_name("new-article");
                get_widget!(self.builder, gtk::Entry, article_url_entry);
                article_url_entry.grab_focus_without_selecting();
                get_widget!(self.builder, gtk::Button, save_article_btn);
                window.set_default_widget(Some(&save_article_btn));
            }
        }
    }

    fn update_size_class(widget: &gtk::ApplicationWindow) {
        if widget.default_width() <= 450 {
            widget.add_css_class("sm");
            widget.remove_css_class("md");
            widget.remove_css_class("lg");
        } else if widget.default_width() <= 600 {
            widget.add_css_class("md");
            widget.remove_css_class("sm");
            widget.remove_css_class("lg");
        } else {
            widget.add_css_class("lg");
            widget.remove_css_class("sm");
            widget.remove_css_class("md");
        }
    }

    fn init(&self) {
        get_widget!(self.builder, gtk::Stack, headerbar_stack);
        get_widget!(self.builder, adw::ViewSwitcherBar, view_switcher_bar);

        headerbar_stack.connect_visible_child_name_notify(move |headerbar_stack| {
            let visible_headerbar_stack = headerbar_stack.visible_child_name().unwrap();
            view_switcher_bar.set_visible(visible_headerbar_stack == "articles");
        });
        self.widget
            .connect_default_width_notify(Self::update_size_class);
        Self::update_size_class(&self.widget);
    }

    fn init_views(&self) {
        get_widget!(self.builder, gtk::Stack, main_stack);
        // Login Form
        main_stack.add_named(&self.login_view, Some("login"));

        // Articles
        get_widget!(self.builder, adw::ViewSwitcherTitle, view_switcher_title);
        get_widget!(self.builder, adw::ViewSwitcherBar, view_switcher_bar);

        main_stack.add_named(&self.articles_view.widget, Some("articles"));
        view_switcher_title.set_stack(Some(&self.articles_view.widget));
        view_switcher_bar.set_stack(Some(&self.articles_view.widget));

        // Article View
        main_stack.add_named(
            self.article_view.get_widget(),
            Some(&self.article_view.name),
        );
        self.widget
            .insert_action_group("article", Some(self.article_view.get_actions()));

        main_stack.connect_visible_child_name_notify(
            clone!(@strong self.article_view as article_view => move |stack| {
                if let Some(view_name) = stack.visible_child_name() {
                    article_view.set_enable_actions(view_name == "article");
                }
            }),
        );

        get_widget!(self.builder, gtk::Button, save_article_btn);
        get_widget!(self.builder, gtk::Entry, article_url_entry);
        save_article_btn.connect_clicked(clone!(@strong self.sender as sender => move |_| {
            if let Ok(url) = Url::parse(&article_url_entry.text()) {
                send!(sender, Action::SaveArticle(url));
                article_url_entry.set_text("");
            }
        }));

        self.set_view(View::Login);
    }

    fn setup_actions(&self) {
        get_widget!(self.builder, gtk::Revealer, notification);

        action!(self.actions, "close-notification", move |_, _| {
            notification.set_reveal_child(false);
        });

        action!(
            self.actions,
            "previous",
            clone!(@strong self.sender as sender => move |_, _| {
                send!(sender, Action::PreviousView);
            })
        );

        self.widget
            .insert_action_group("window", Some(&self.actions));
    }
}
