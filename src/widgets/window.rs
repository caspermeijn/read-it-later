use failure::Error;
use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use libhandy::prelude::*;
use libhandy::SearchBarExt;
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

use crate::application::Action;
use crate::config::{APP_ID, PROFILE};
use crate::models::Article;
use crate::views::{ArchiveView, ArticleView, FavoritesView, LoginView, UnreadView};
use crate::window_state;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum View {
    Article,       // Article
    Login,         // Sign in
    Unread,        // Unread articles
    Archive,       // Archived articles
    Favorites,     // Favorites articles
    Syncing(bool), // During sync
    NewArticle,    // New Article
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    builder: gtk::Builder,
    sender: Sender<Action>,
    pub view_history: Rc<RefCell<Vec<View>>>,
    article_view: ArticleView,
    unread_view: UnreadView,
    favorites_view: FavoritesView,
    archive_view: ArchiveView,
    actions: gio::SimpleActionGroup,
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Rc<Self> {
        let settings = gio::Settings::new(APP_ID);
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/window.ui");
        get_widget!(builder, gtk::ApplicationWindow, window);

        if PROFILE == "Devel" {
            window.get_style_context().add_class("devel");
        }
        let actions = gio::SimpleActionGroup::new();

        let window_widget = Rc::new(Window {
            widget: window,
            builder,
            view_history: Rc::new(RefCell::new(vec![])),
            article_view: ArticleView::new(sender.clone()),
            unread_view: UnreadView::new(sender.clone()),
            favorites_view: FavoritesView::new(sender.clone()),
            archive_view: ArchiveView::new(sender.clone()),
            sender,
            actions,
        });

        window_widget.init(settings);
        window_widget.init_views(window_widget.clone());
        window_widget.setup_actions();
        window_widget
    }

    pub fn add_article(&self, article: Article) {
        if article.is_starred {
            self.favorites_view.add(article);
        } else if article.is_archived {
            self.archive_view.add(article);
        } else {
            self.unread_view.add(article);
        }
    }

    pub fn update_article(&self, article: Article) {}

    pub fn delete_article(&self, article: Article) -> Result<(), Error> {
        if !article.is_starred && !article.is_archived {
            self.unread_view.delete(article.clone());
        } else {
            if article.is_starred {
                self.favorites_view.delete(article.clone());
            }
            if article.is_archived {
                self.archive_view.delete(article.clone());
            }
        }
        Ok(article.delete()?)
    }

    pub fn favorite_article(&self, article: Article) {
        if !article.is_starred {
            if !article.is_archived {
                self.unread_view.add(article.clone());
            }
            self.favorites_view.delete(article.clone());
        } else {
            self.favorites_view.add(article.clone());
            self.unread_view.delete(article.clone());
        }
    }

    pub fn archive_article(&self, article: Article) {
        if !article.is_archived {
            if !article.is_starred {
                self.unread_view.add(article.clone());
            }
            self.archive_view.delete(article.clone());
        } else {
            self.archive_view.add(article.clone());
            self.unread_view.delete(article.clone());
        }
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

    pub fn get_new_article_url(&self) -> Option<Url> {
        get_widget!(self.builder, gtk::Entry, article_url_entry);

        if let Ok(url) = Url::parse(&article_url_entry.get_text().unwrap()) {
            return Some(url);
        }
        return None;
    }

    pub fn notify(&self, message: String) {
        get_widget!(self.builder, gtk::Revealer, notification);
        get_widget!(self.builder, gtk::Label, notification_label);

        notification_label.set_text(&message);
        notification.set_reveal_child(true);

        gtk::timeout_add_seconds(3, move || {
            notification.set_reveal_child(false);
            glib::Continue(false)
        });
    }

    pub fn set_view(&self, view: View) {
        get_widget!(self.builder, gtk::Stack, main_stack);
        get_widget!(self.builder, gtk::Stack, headerbar_stack);
        match view {
            View::Article => {
                main_stack.set_visible_child_name("article");
                headerbar_stack.set_visible_child_name("article");
            }
            View::Archive => {
                self.archive_view.get_widget().queue_resize();
                main_stack.set_visible_child_name("archive");
                headerbar_stack.set_visible_child_name("articles");
            }
            View::Favorites => {
                self.favorites_view.get_widget().queue_resize();
                main_stack.set_visible_child_name("favorites");
                headerbar_stack.set_visible_child_name("articles");
            }
            View::Unread => {
                self.unread_view.get_widget().queue_resize();
                main_stack.set_visible_child_name("unread");
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
        // Store the view in history
        match view {
            View::Article | View::Unread | View::Favorites | View::Archive | View::NewArticle => {
                let view_history = self.view_history.borrow().clone();
                match view_history.last() {
                    Some(v) => {
                        if v != &view {
                            // Avoid saving the same view twice
                            self.view_history.borrow_mut().push(view);
                        }
                    }
                    _ => self.view_history.borrow_mut().push(view),
                }
            }
            _ => (),
        }
    }

    pub fn previous_view(&self) {
        get_widget!(self.builder, libhandy::SearchBar, searchbar);
        if searchbar.get_search_mode() {
            // Just disable search
            searchbar.set_search_mode(false);
            return;
        }

        let total_views = self.view_history.borrow().len();
        if total_views < 2 {
            return; // No previous view available
        }
        let current_view = self.view_history.borrow_mut().pop(); // Remove current view from history
        if current_view == Some(View::Article) {
            searchbar.set_search_mode(false);
        }

        let new_view = self.view_history.borrow().clone();
        if let Some(view) = new_view.get(new_view.len() - 1) {
            self.set_view(*view);
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

    fn init_views(&self, win: Rc<Self>) {
        get_widget!(self.builder, gtk::Stack, main_stack);
        // Login Form
        let login_view = LoginView::new(self.sender.clone());
        main_stack.add_named(&login_view.get_widget(), &login_view.name);

        // Unread View
        main_stack.add_titled(&self.unread_view.get_widget(), &self.unread_view.name, &self.unread_view.title);
        main_stack.set_child_icon_name(&self.unread_view.get_widget(), Some(&self.unread_view.icon));

        // Favorites View
        main_stack.add_titled(
            &self.favorites_view.get_widget(),
            &self.favorites_view.name,
            &self.favorites_view.title,
        );
        main_stack.set_child_icon_name(&self.favorites_view.get_widget(), Some(&self.favorites_view.icon));

        // Archive View
        main_stack.add_titled(&self.archive_view.get_widget(), &self.archive_view.name, &self.archive_view.title);
        main_stack.set_child_icon_name(&self.archive_view.get_widget(), Some(&self.archive_view.icon));

        // Article View
        main_stack.add_named(&self.article_view.get_widget(), &self.article_view.name);
        self.widget.insert_action_group("article", self.article_view.get_actions());

        // hackish way to sync unread/favorites/archive with view history :p
        let article_view = self.article_view.clone();

        main_stack.connect_property_visible_child_name_notify(move |stack| {
            if let Some(view_name) = stack.get_visible_child_name() {
                if view_name == "unread" {
                    win.set_view(View::Unread);
                } else if view_name == "favorites" {
                    win.set_view(View::Favorites);
                } else if view_name == "archive" {
                    win.set_view(View::Archive);
                }
                article_view.set_enable_actions(view_name == "article");
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
