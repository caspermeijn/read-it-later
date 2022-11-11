use crate::models::{Article, ArticleAction};
use anyhow::Result;
use glib::subclass::InitializingObject;
use glib::Object;
use gtk::gio::prelude::*;
use gtk::glib::clone;
use gtk::glib::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};
use gtk_macros::{action, send, stateful_action};
use log::{error, info};
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use webkit::traits::{ContextMenuExt, ContextMenuItemExt, WebViewExt};
use webkit::Settings;
use webkit::WebView;

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
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
            WebView::ensure_type();
            Settings::ensure_type();
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticleWidget {
        fn dispose(&self) {
            while let Some(child) = self.instance().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ArticleWidget {}
}

glib::wrapper! {
    pub struct ArticleWidget(ObjectSubclass<imp::ArticleWidget>)
        @extends gtk::Widget;
}

impl ArticleWidget {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let article_widget: Self = Object::new(&[]);
        article_widget.init(sender);
        article_widget.setup_actions();
        article_widget
    }

    fn init(&self, sender: Sender<ArticleAction>) {
        self.imp().sender.set(sender).unwrap();

        // Right/Left Click context menu
        let forbidden_actions = vec![
            webkit::ContextMenuAction::OpenLink,
            webkit::ContextMenuAction::GoBack,
            webkit::ContextMenuAction::GoForward,
            webkit::ContextMenuAction::Stop,
            webkit::ContextMenuAction::Reload,
            webkit::ContextMenuAction::InspectElement,
        ];

        self.imp().webview.connect_context_menu(move |_, context_menu, _, _| {
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
        self.imp()
            .webview
            .connect_estimated_load_progress_notify(clone!(@strong self as aw => move |webview|{
                let progress = webview.estimated_load_progress();
                aw.imp().revealer.set_reveal_child(true);
                aw.imp().progressbar.set_fraction(progress);
                if (progress - 1.0).abs() < std::f64::EPSILON {
                    aw.imp().revealer.set_reveal_child(false);
                }
            }));
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
        stateful_action!(
            self.imp().actions,
            "archive",
            false,
            clone!(@strong self as aw, @strong sender => move |action, _|{
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_archived = !action_state;
                action.set_state(&is_archived.to_variant());
                if let Some(article) = aw.imp().article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Archive(article));
                }
            })
        );
        // Favorite article
        stateful_action!(
            self.imp().actions,
            "favorite",
            false,
            clone!(@strong self as aw, @strong sender => move |action, _|{
                let state = action.state().unwrap();
                let action_state: bool = state.get().unwrap();
                let is_starred = !action_state;
                action.set_state(&is_starred.to_variant());

                if let Some(article) = aw.imp().article.borrow_mut().clone() {
                    send!(sender, ArticleAction::Favorite(article));
                }
            })
        );
    }

    pub fn load_article(&self, article: Article) -> Result<()> {
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
}

pub fn load_resource(file: &str) -> Result<String> {
    let file = gio::File::for_uri(&format!("resource:///com/belmoussaoui/ReadItLater/{}", file));
    let (bytes, _) = file.load_bytes(gio::Cancellable::NONE)?;
    String::from_utf8(bytes.to_vec()).map_err(From::from)
}
