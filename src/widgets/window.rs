use glib::Sender;
use gtk::prelude::*;
use libhandy::prelude::*;

use crate::application::Action;
use crate::config::{APP_ID, PROFILE};
use crate::models::Article;
use crate::views::{ArchiveView, ArticleView, FavoritesView, LoginView, SyncingView, UnreadView};
use crate::window_state;

pub enum View {
    Article,   // Article
    Login,     // Sign in
    Error,     // Network & other errors
    Unread,    // Unread articles
    Archive,   // Archived articles
    Favorites, // Favorites articles
    Syncing,   // During sync
}

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    builder: gtk::Builder,
    sender: Sender<Action>,
    article_view: ArticleView,
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let settings = gio::Settings::new(APP_ID);
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/window.ui");
        let widget: gtk::ApplicationWindow = builder.get_object("window").expect("Failed to retrieve Window");
        if PROFILE == "Devel" {
            widget.get_style_context().add_class("devel");
        }

        let window = Window {
            widget,
            builder,
            article_view: ArticleView::new(sender.clone()),
            sender,
        };

        window.init(settings);
        window.init_views();
        window
    }

    pub fn load_article(&self, article: Article) {
        self.article_view.load(article);
        self.set_view(View::Article);
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

        self.widget.connect_size_allocate(move |_, allocation| {
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

    fn init_views(&self) {
        let main_stack: gtk::Stack = self.builder.get_object("main_stack").expect("Failed to retrieve main_stack");
        // Login Form
        let login_view = LoginView::new(self.sender.clone());
        main_stack.add_named(&login_view.get_widget(), &login_view.name);

        // Syncing View: Spinner + Loading message
        let syncing_view = SyncingView::new(self.sender.clone());
        main_stack.add_named(&syncing_view.get_widget(), &syncing_view.name);

        // Unread View
        let unread_view = UnreadView::new(self.sender.clone());
        main_stack.add_titled(&unread_view.get_widget(), &unread_view.name, &unread_view.title);
        main_stack.set_child_icon_name(&unread_view.get_widget(), Some(&unread_view.icon));

        // Favorites View
        let favorites_view = FavoritesView::new(self.sender.clone());
        main_stack.add_titled(&favorites_view.get_widget(), &favorites_view.name, &favorites_view.title);
        main_stack.set_child_icon_name(&favorites_view.get_widget(), Some(&favorites_view.icon));

        // Archive View
        let archive_view = ArchiveView::new(self.sender.clone());
        main_stack.add_titled(&archive_view.get_widget(), &archive_view.name, &archive_view.title);
        main_stack.set_child_icon_name(&archive_view.get_widget(), Some(&archive_view.icon));

        // Article View
        main_stack.add_named(&self.article_view.get_widget(), &self.article_view.name);

        self.set_view(View::Login);
    }

    pub fn set_view(&self, view: View) {
        let main_stack: gtk::Stack = self.builder.get_object("main_stack").expect("Failed to retrieve main_stack");
        let headerbar_stack: gtk::Stack = self.builder.get_object("headerbar_stack").expect("Failed to retrieve headerbar_stack");

        let squeezer: libhandy::Squeezer = self.builder.get_object("squeezer").unwrap();
        let switcher_bar: libhandy::ViewSwitcherBar = self.builder.get_object("switcher_bar").unwrap();

        let title_label: gtk::Label = self.builder.get_object("title_label").unwrap();

        match view {
            View::Article => {
                main_stack.set_visible_child_name("article");
                headerbar_stack.set_visible_child_name("article");
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
        }
    }
}
