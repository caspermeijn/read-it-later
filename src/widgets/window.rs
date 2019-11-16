use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use libhandy::prelude::*;
use libhandy::SearchBarExt;
use url::Url;

use crate::application::Action;
use crate::config::{APP_ID, PROFILE};
use crate::models::{Article, ArticlesManager};
use crate::views::{ArticleView, ArticlesView, LoginView};
use crate::window_state;

#[derive(Copy, Clone, Debug, PartialEq)]
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
    login_view: LoginView,
    actions: gio::SimpleActionGroup,
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let settings = gio::Settings::new(APP_ID);
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/window.ui");
        get_widget!(builder, gtk::ApplicationWindow, window);

        if PROFILE == "Devel" {
            window.get_style_context().add_class("devel");
        }
        let actions = gio::SimpleActionGroup::new();

        let articles_manager = ArticlesManager::new(sender.clone());

        let window_widget = Window {
            widget: window,
            builder,
            article_view: ArticleView::new(articles_manager.sender.clone()),
            articles_view: ArticlesView::new(articles_manager.sender.clone()),
            login_view: LoginView::new(sender.clone()),
            sender,
            actions,
        };

        window_widget.init(settings);
        window_widget.init_views();
        window_widget.setup_actions();
        window_widget
    }

    pub fn load_article(&self, article: Article) {
        if let Some(article_view_actions) = self.article_view.get_actions() {
            let archive_action = article_view_actions
                .lookup_action("archive")
                .unwrap()
                .downcast::<gio::SimpleAction>()
                .unwrap();
            let favorite_action = article_view_actions
                .lookup_action("favorite")
                .unwrap()
                .downcast::<gio::SimpleAction>()
                .unwrap();

            favorite_action.set_state(&article.is_starred.to_variant());
            archive_action.set_state(&article.is_archived.to_variant());
        }

        self.article_view.load(article);
        self.set_view(View::Article);
    }

    pub fn notify(&self, message: String) {
        get_widget!(self.builder, gtk::Revealer, notification);
        get_widget!(self.builder, gtk::Label, notification_label);

        notification_label.set_text(&message);
        notification.set_reveal_child(true);

        gtk::timeout_add_seconds(5, move || {
            notification.set_reveal_child(false);
            glib::Continue(false)
        });
    }

    pub fn previous_view(&self) {
        self.set_view(View::Articles);
    }

    pub fn set_view(&self, view: View) {
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
            }
            View::Syncing(state) => {
                get_widget!(self.builder, gtk::ProgressBar, loading_progress);
                loading_progress.set_visible(state);
                if !state {
                    // If we hide the progress bar
                    loading_progress.set_fraction(0.0); // Reset the fraction
                } else {
                    loading_progress.pulse();
                    gtk::timeout_add(200, move || {
                        loading_progress.pulse();
                        glib::Continue(loading_progress.get_visible())
                    });
                }
            }
            View::NewArticle => {
                headerbar_stack.set_visible_child_name("new-article");
                get_widget!(self.builder, gtk::Entry, article_url_entry);
                article_url_entry.grab_focus_without_selecting();
            }
        }
    }

    fn init(&self, settings: gio::Settings) {
        // setup app menu
        let menu_builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/menu.ui");
        get_widget!(menu_builder, gtk::PopoverMenu, popover_menu);
        get_widget!(self.builder, gtk::MenuButton, appmenu_button);
        appmenu_button.set_popover(Some(&popover_menu));
        // load latest window state
        window_state::load(&self.widget, &settings);

        // save window state on delete event
        self.widget.connect_delete_event(move |window, _| {
            window_state::save(&window, &settings);
            Inhibit(false)
        });

        get_widget!(self.builder, libhandy::Squeezer, squeezer);
        get_widget!(self.builder, gtk::Stack, headerbar_stack);
        get_widget!(self.builder, libhandy::ViewSwitcher, title_wide_switcher);
        get_widget!(self.builder, libhandy::ViewSwitcher, title_narrow_switcher);
        get_widget!(self.builder, libhandy::ViewSwitcherBar, switcher_bar);
        get_widget!(self.builder, gtk::Label, title_label);

        self.widget.connect_size_allocate(move |widget, allocation| {
            if allocation.width <= 450 {
                widget.get_style_context().add_class("sm");
                widget.get_style_context().remove_class("md");
                widget.get_style_context().remove_class("lg");
            } else if allocation.width <= 600 {
                widget.get_style_context().add_class("md");
                widget.get_style_context().remove_class("sm");
                widget.get_style_context().remove_class("lg");
            } else {
                widget.get_style_context().add_class("lg");
                widget.get_style_context().remove_class("sm");
                widget.get_style_context().remove_class("md");
            }
            if headerbar_stack.get_visible_child_name() == Some("articles".into()) {
                squeezer.set_child_enabled(&title_wide_switcher, allocation.width > 600);
                squeezer.set_child_enabled(&title_label, allocation.width <= 450);
                squeezer.set_child_enabled(&title_narrow_switcher, allocation.width > 450);
                switcher_bar.set_reveal(allocation.width <= 450);
            } else {
                switcher_bar.set_reveal(false);
            }
        });

        get_widget!(self.builder, libhandy::SearchBar, searchbar);
        get_widget!(self.builder, gtk::ModelButton, search_button);
        search_button.connect_clicked(clone!(searchbar => move |_| {
            searchbar.set_search_mode(true);
        }));

        get_widget!(self.builder, gtk::ToggleButton, search_togglebutton);
        searchbar
            .bind_property("search-mode-enabled", &search_togglebutton, "active")
            .flags(glib::BindingFlags::SYNC_CREATE)
            .flags(glib::BindingFlags::BIDIRECTIONAL)
            .build();

        get_widget!(self.builder, gtk::SearchEntry, search_entry);
        searchbar.connect_property_search_mode_enabled_notify(move |search_bar| {
            if search_bar.get_search_mode() {
                search_entry.grab_focus_without_selecting();
            }
        });
    }

    fn init_views(&self) {
        get_widget!(self.builder, gtk::Stack, main_stack);
        // Login Form
        main_stack.add_named(&self.login_view.get_widget(), &self.login_view.name);

        // Articles
        get_widget!(self.builder, libhandy::ViewSwitcher, title_wide_switcher);
        get_widget!(self.builder, libhandy::ViewSwitcher, title_narrow_switcher);
        get_widget!(self.builder, libhandy::ViewSwitcherBar, switcher_bar);

        main_stack.add_named(&self.articles_view.widget, "articles");
        title_wide_switcher.set_stack(Some(&self.articles_view.widget));
        title_narrow_switcher.set_stack(Some(&self.articles_view.widget));
        switcher_bar.set_stack(Some(&self.articles_view.widget));

        // Article View
        main_stack.add_named(&self.article_view.get_widget(), &self.article_view.name);
        self.widget.insert_action_group("article", self.article_view.get_actions());

        let article_view = self.article_view.clone();
        main_stack.connect_property_visible_child_name_notify(move |stack| {
            if let Some(view_name) = stack.get_visible_child_name() {
                article_view.set_enable_actions(view_name == "article");
            }
        });

        let sender = self.sender.clone();
        get_widget!(self.builder, gtk::Button, save_article_btn);
        get_widget!(self.builder, gtk::Entry, article_url_entry);
        save_article_btn.connect_clicked(move |_| {
            if let Ok(url) = Url::parse(&article_url_entry.get_text().unwrap()) {
                send!(sender, Action::SaveArticle(url));
            }
        });
    }

    fn setup_actions(&self) {
        get_widget!(self.builder, gtk::Revealer, notification);
        let simple_action = gio::SimpleAction::new("close-notification", None);
        simple_action.connect_activate(move |_, _| {
            notification.set_reveal_child(false);
        });
        self.actions.add_action(&simple_action);

        self.widget.insert_action_group("window", Some(&self.actions));
    }
}
