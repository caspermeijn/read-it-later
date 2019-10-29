use failure::Error;
use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;
use libhandy::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use url::Url;

use crate::application::Action;
use crate::config::{APP_ID, PROFILE};
use crate::models::Article;
use crate::views::{ArchiveView, ArticleView, FavoritesView, LoginView, SyncingView, UnreadView};
use crate::window_state;

#[derive(Copy, Clone, Debug)]
pub enum View {
    Article,    // Article
    Login,      // Sign in
    Error,      // Network & other errors
    Unread,     // Unread articles
    Archive,    // Archived articles
    Favorites,  // Favorites articles
    Syncing,    // During sync
    NewArticle, // New Article
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
        let widget: gtk::ApplicationWindow = builder.get_object("window").expect("Failed to retrieve Window");
        if PROFILE == "Devel" {
            widget.get_style_context().add_class("devel");
        }
        let actions = gio::SimpleActionGroup::new();

        let window = Rc::new(Window {
            widget,
            builder,
            view_history: Rc::new(RefCell::new(vec![View::Login])),
            article_view: ArticleView::new(sender.clone()),
            unread_view: UnreadView::new(sender.clone()),
            favorites_view: FavoritesView::new(sender.clone()),
            archive_view: ArchiveView::new(sender.clone()),
            sender,
            actions,
        });

        window.init(settings);
        window.init_views(window.clone());
        window.setup_actions();
        window
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

    pub fn favorite_article(&self, mut article: Article) -> Result<(), Error> {
        article.toggle_favorite()?;
        if !article.is_starred {
            if !article.is_archived {
                self.unread_view.add(article.clone());
            }
            self.favorites_view.delete(article.clone());
        } else {
            self.favorites_view.add(article.clone());
            self.unread_view.delete(article.clone());
        }
        Ok(())
    }

    pub fn archive_article(&self, mut article: Article) -> Result<(), Error> {
        article.toggle_archive()?;
        if !article.is_archived {
            if !article.is_starred {
                self.unread_view.add(article.clone());
            }
            self.archive_view.delete(article.clone());
        } else {
            self.archive_view.add(article.clone());
            self.unread_view.delete(article.clone());
        }
        Ok(())
    }

    pub fn load_article(&self, article: Article) {
        get_widget!(self.builder, gtk::ToggleButton, archive_togglebtn);
        get_widget!(self.builder, gtk::ToggleButton, favorite_togglebtn);

        if let Some(article_view_actions) = self.article_view.get_actions() {
            let archive_action = article_view_actions.lookup_action("archive").unwrap().downcast::<gio::SimpleAction>().unwrap();
            let favorite_action = article_view_actions.lookup_action("favorite").unwrap().downcast::<gio::SimpleAction>().unwrap();

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
        get_widget!(self.builder, gtk::HeaderBar, headerbar);
        get_widget!(self.builder, gtk::Stack, headerbar_stack);

        headerbar.set_show_close_button(true);

        self.article_view.set_enable_actions(false);

        match view {
            View::Article => {
                self.article_view.set_enable_actions(true);
                main_stack.set_visible_child_name("article");
                headerbar_stack.set_visible_child_name("article");
                self.widget.insert_action_group("article", self.article_view.get_actions());
            }
            View::Archive => {
                main_stack.set_visible_child_name("archive");
                headerbar_stack.set_visible_child_name("articles");
            }
            View::Favorites => {
                main_stack.set_visible_child_name("favorites");
                headerbar_stack.set_visible_child_name("articles");
            }
            View::Unread => {
                main_stack.set_visible_child_name("unread");
                headerbar_stack.set_visible_child_name("articles");
            }
            View::Error => (),
            View::Login => {
                main_stack.set_visible_child_name("login");
                headerbar_stack.set_visible_child_name("login");
            }
            View::Syncing => {
                main_stack.set_visible_child_name("syncing");
            }
            View::NewArticle => {
                headerbar_stack.set_visible_child_name("new-article");
                headerbar.set_show_close_button(false);
            }
        }
        if self.view_history.borrow().len() == 3 {
            self.view_history.borrow_mut().remove(0); // remove the oldest element
        }
        self.view_history.borrow_mut().push(view);
    }

    pub fn previous_view(&self) {
        // We support only one step back in time
        let view_history = self.view_history.borrow().clone();
        let total_views = view_history.len();
        if let Some(view) = view_history.get(total_views - 2) {
            self.set_view(*view);
        }
    }

    fn init(&self, settings: gio::Settings) {
        // setup app menu
        let menu_builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/menu.ui");
        let popover_menu: gtk::PopoverMenu = menu_builder.get_object("popover_menu").expect("Failed to retrieve the popover");
        let appmenu_btn: gtk::MenuButton = self.builder.get_object("appmenu_button").expect("Failed to retrive the primary menu");
        appmenu_btn.set_popover(Some(&popover_menu));
        // load latest window state
        window_state::load(&self.widget, &settings);

        // save window state on delete event
        self.widget.connect_delete_event(move |window, _| {
            window_state::save(&window, &settings);
            Inhibit(false)
        });

        let squeezer: libhandy::Squeezer = self.builder.get_object("squeezer").unwrap();
        let switcher_bar: libhandy::ViewSwitcherBar = self.builder.get_object("switcher_bar").unwrap();
        let headerbar_stack: gtk::Stack = self.builder.get_object("headerbar_stack").expect("Failed to retrieve headerbar_stack");

        let title_wide_switcher: libhandy::ViewSwitcher = self.builder.get_object("title_wide_switcher").unwrap();
        let title_narrow_switcher: libhandy::ViewSwitcher = self.builder.get_object("title_narrow_switcher").unwrap();
        let title_label: gtk::Label = self.builder.get_object("title_label").unwrap();

        self.widget.connect_size_allocate(move |widget, allocation| {
            if allocation.width <= 450 {
                widget.get_style_context().add_class("sm");
                widget.get_style_context().remove_class("md");
                widget.get_style_context().remove_class("lg");
            } else if allocation.width <= 850 {
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
    }

    fn init_views(&self, win: Rc<Self>) {
        let main_stack: gtk::Stack = self.builder.get_object("main_stack").expect("Failed to retrieve main_stack");
        // Login Form
        let login_view = LoginView::new(self.sender.clone());
        main_stack.add_named(&login_view.get_widget(), &login_view.name);

        // Syncing View: Spinner + Loading message
        let syncing_view = SyncingView::new(self.sender.clone());
        main_stack.add_named(&syncing_view.get_widget(), &syncing_view.name);

        // Unread View
        main_stack.add_titled(&self.unread_view.get_widget(), &self.unread_view.name, &self.unread_view.title);
        main_stack.set_child_icon_name(&self.unread_view.get_widget(), Some(&self.unread_view.icon));

        // Favorites View
        main_stack.add_titled(&self.favorites_view.get_widget(), &self.favorites_view.name, &self.favorites_view.title);
        main_stack.set_child_icon_name(&self.favorites_view.get_widget(), Some(&self.favorites_view.icon));

        // Archive View
        main_stack.add_titled(&self.archive_view.get_widget(), &self.archive_view.name, &self.archive_view.title);
        main_stack.set_child_icon_name(&self.archive_view.get_widget(), Some(&self.archive_view.icon));

        // Article View
        main_stack.add_named(&self.article_view.get_widget(), &self.article_view.name);
        self.widget.insert_action_group("article", self.article_view.get_actions());

        // hackish way to sync unread/favorites/archive with view history :p
        main_stack.connect_property_visible_child_name_notify(move |stack| {
            if let Some(view_name) = stack.get_visible_child_name() {
                if view_name == "unread" {
                    win.set_view(View::Unread);
                } else if view_name == "favorites" {
                    win.set_view(View::Favorites);
                } else if view_name == "archive" {
                    win.set_view(View::Archive);
                }
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
