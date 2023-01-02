use adw::prelude::*;
use glib::{clone, timeout_future_seconds, MainContext, Object, Sender};
use gtk::{gio, glib, subclass::prelude::*};
use gtk_macros::{action, get_action, send};
use log::error;
use url::Url;

use crate::{
    application::Action,
    config::PROFILE,
    models::{Article, ArticlesManager},
    views::{ArticleView, ArticlesView, Login},
};

mod imp {
    use adw::subclass::prelude::*;
    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use once_cell::sync::OnceCell;

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/window.ui")]
    pub struct Window {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub headerbar_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub loading_progress: TemplateChild<gtk::ProgressBar>,
        #[template_child]
        pub article_url_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub save_article_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub view_switcher_bar: TemplateChild<adw::ViewSwitcherBar>,
        #[template_child]
        pub view_switcher_title: TemplateChild<adw::ViewSwitcherTitle>,
        pub sender: OnceCell<Sender<Action>>,
        pub article_view: OnceCell<ArticleView>,
        pub articles_view: OnceCell<ArticlesView>,
        pub login_view: Login,
        pub actions: gio::SimpleActionGroup,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Window {
        const NAME: &'static str = "Window";
        type Type = super::Window;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Window {
        fn dispose(&self) {
            self.obj().dispose_template(Self::Type::static_type());
        }
    }

    impl WidgetImpl for Window {}

    impl WindowImpl for Window {}

    impl ApplicationWindowImpl for Window {}

    impl AdwApplicationWindowImpl for Window {}
}

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
    @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum View {
    Article,       // Article
    Login,         // Sign in
    Articles,      // Unread articles
    Syncing(bool), // During sync
    NewArticle,    // New Article
}

impl Window {
    pub fn new(sender: Sender<Action>) -> Self {
        let window: Self = Object::new(&[]);
        window.init(sender);
        window
    }

    pub fn load_article(&self, article: Article) {
        let article_view = self.imp().article_view.get().unwrap();
        let article_view_actions = article_view.get_actions();
        get_action!(article_view_actions, @archive).set_state(&article.is_archived.to_variant());
        get_action!(article_view_actions, @favorite).set_state(&article.is_starred.to_variant());
        article_view.load(article);
        self.set_view(View::Article);
    }

    pub fn add_toast(&self, toast: adw::Toast) {
        self.imp().toast_overlay.add_toast(&toast);
    }

    pub fn previous_view(&self) {
        self.set_view(View::Articles);
    }

    pub fn set_view(&self, view: View) {
        let imp = self.imp();
        self.set_default_widget(gtk::Widget::NONE);
        match view {
            View::Article => {
                imp.main_stack.set_visible_child_name("article");
                imp.headerbar_stack.set_visible_child_name("article");
            }
            View::Articles => {
                imp.main_stack.set_visible_child_name("articles");
                imp.headerbar_stack.set_visible_child_name("articles");
            }
            View::Login => {
                imp.main_stack.set_visible_child_name("login");
                imp.headerbar_stack.set_visible_child_name("login");

                self.set_default_widget(Some(imp.login_view.get_login_button()));
            }
            View::Syncing(state) => {
                imp.loading_progress.set_visible(state);
                if !state {
                    // If we hide the progress bar
                    imp.loading_progress.set_fraction(0.0); // Reset the fraction
                } else {
                    let main_context = MainContext::default();

                    imp.loading_progress.pulse();

                    let future = clone!(@weak imp => async move {
                        timeout_future_seconds(1).await;
                        imp.loading_progress.pulse();
                    });

                    main_context.spawn_local(future);
                }
            }
            View::NewArticle => {
                imp.headerbar_stack.set_visible_child_name("new-article");
                imp.article_url_entry.grab_focus_without_selecting();
                self.set_default_widget(Some(&imp.save_article_btn.get()));
            }
        }
    }

    fn update_size_class(widget: &Self) {
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

    pub fn init(&self, sender: Sender<Action>) {
        let imp = self.imp();

        if PROFILE == "Devel" {
            self.add_css_class("devel");
        }
        let articles_manager = ArticlesManager::new(sender.clone());

        imp.sender.set(sender).unwrap();
        imp.article_view
            .set(ArticleView::new(articles_manager.sender.clone()))
            .unwrap();
        imp.articles_view
            .set(ArticlesView::new(articles_manager.sender.clone()))
            .unwrap();

        imp.headerbar_stack.connect_visible_child_name_notify(
            clone!(@weak imp => move |headerbar_stack| {
                let visible_headerbar_stack = headerbar_stack.visible_child_name().unwrap();
                imp.view_switcher_bar
                    .set_visible(visible_headerbar_stack == "articles");
            }),
        );
        self.connect_default_width_notify(Self::update_size_class);
        Self::update_size_class(&self);

        self.init_views();
        self.setup_actions();
    }

    fn init_views(&self) {
        let imp = self.imp();

        // Login Form
        imp.main_stack.add_named(&imp.login_view, Some("login"));

        // Articles
        let articles_view = imp.articles_view.get().unwrap();
        imp.main_stack
            .add_named(&articles_view.widget, Some("articles"));
        imp.view_switcher_title
            .set_stack(Some(&articles_view.widget));
        imp.view_switcher_bar.set_stack(Some(&articles_view.widget));

        // Article View
        let article_view = imp.article_view.get().unwrap();
        imp.main_stack
            .add_named(article_view.get_widget(), Some(&article_view.name));
        self.insert_action_group("article", Some(article_view.get_actions()));

        imp.main_stack.connect_visible_child_name_notify(
            clone!(@strong article_view => move |stack| {
                if let Some(view_name) = stack.visible_child_name() {
                    article_view.set_enable_actions(view_name == "article");
                }
            }),
        );

        let sender = imp.sender.get().unwrap();
        imp.save_article_btn
            .connect_clicked(clone!(@weak imp, @strong sender => move |_| {
                if let Ok(url) = Url::parse(&imp.article_url_entry.text()) {
                    send!(sender, Action::SaveArticle(url));
                    imp.article_url_entry.set_text("");
                }
            }));

        self.set_view(View::Login);
    }

    fn setup_actions(&self) {
        let imp = self.imp();

        let sender = imp.sender.get().unwrap();
        action!(
            imp.actions,
            "previous",
            clone!(@strong sender => move |_, _| {
                send!(sender, Action::PreviousView);
            })
        );

        self.insert_action_group("window", Some(&imp.actions));
    }

    pub fn articles_view(&self) -> &ArticlesView {
        self.imp().articles_view.get().unwrap()
    }
}
