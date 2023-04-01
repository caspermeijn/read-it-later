use std::cell::RefCell;

use anyhow::Result;
use glib::{clone, Sender};
use gtk::{gio, glib, prelude::*, subclass::prelude::*};
use gtk_macros::{action, get_action, send};
use log::{error, info};
use once_cell::sync::OnceCell;
use webkit::{prelude::*, Settings, WebView};

use crate::models::{Article, ArticleAction};

mod imp {
    use glib::subclass::InitializingObject;

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article.ui")]
    pub struct ArticleView {
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
    impl ObjectSubclass for ArticleView {
        const NAME: &'static str = "ArticleView";
        type Type = super::ArticleView;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            Settings::ensure_type();
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticleView {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticleView {}

    #[gtk::template_callbacks]
    impl ArticleView {
        #[template_callback]
        fn modify_context_menu(
            _: &WebView,
            context_menu: &webkit::ContextMenu,
            _: &glib::Value,
            _: &webkit::HitTestResult,
        ) -> bool {
            // Right/Left Click context menu
            let forbidden_actions = vec![
                webkit::ContextMenuAction::OpenLink,
                webkit::ContextMenuAction::GoBack,
                webkit::ContextMenuAction::GoForward,
                webkit::ContextMenuAction::Stop,
                webkit::ContextMenuAction::Reload,
                webkit::ContextMenuAction::InspectElement,
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
    pub struct ArticleView(ObjectSubclass<imp::ArticleView>)
        @extends gtk::Widget;
}

impl ArticleView {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_sender(&self, sender: Sender<ArticleAction>) {
        self.imp().sender.set(sender).unwrap();
        self.setup_actions();
    }

    fn setup_actions(&self) {
        let sender = self.imp().sender.get().unwrap();
        // Delete article
        action!(
            self.imp().actions,
            "delete",
            clone!(@strong self as aw, @strong sender => move |_, _| {
                if let Some(article) = aw.imp().article.borrow().clone() {
                    send!(sender, ArticleAction::Delete(article));
                }
            })
        );
        // Share article
        action!(
            self.imp().actions,
            "open",
            clone!(@strong self as aw => move |_, _| {
                if let Some(article) = aw.imp().article.borrow().clone() {
                    glib::idle_add(clone!(@strong article => move || {
                        let article_url = article.url.clone();
                        gtk::show_uri(gtk::Window::NONE, &article_url.unwrap(), 0);
                        glib::Continue(false)
                    }));
                }
            })
        );

        // Archive article
        let simple_action = gio::SimpleAction::new_stateful("archive", None, false.into());
        simple_action.connect_activate(
            clone!(@strong self as aw, @strong sender => move |action, _|{
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_archived = !action_state;
                action.set_state(is_archived.into());
                if let Some(article) = aw.imp().article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Archive(article));
                }
            }),
        );
        self.imp().actions.add_action(&simple_action);

        // Favorite article
        let simple_action = gio::SimpleAction::new_stateful("favorite", None, false.into());
        simple_action.connect_activate(
            clone!(@strong self as aw, @strong sender => move |action, _|{
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_starred = !action_state;
                action.set_state(is_starred.into());

                if let Some(article) = aw.imp().article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Favorite(article));
                }
            }),
        );
        self.imp().actions.add_action(&simple_action);
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

    pub fn set_enable_actions(&self, state: bool) {
        let action_group = self.get_actions();
        get_action!(action_group, @open).set_enabled(state);
        get_action!(action_group, @archive).set_enabled(state);
        get_action!(action_group, @delete).set_enabled(state);
        get_action!(action_group, @favorite).set_enabled(state);
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
