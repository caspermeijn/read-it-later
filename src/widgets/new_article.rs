// Copyright 2023 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::{prelude::*, subclass::prelude::*};
use async_std::channel::Sender;
use glib::Object;
use gtk::glib;
use url::Url;

use crate::application::Action;

mod imp {
    use std::cell::OnceCell;

    use glib::subclass::InitializingObject;
    use gtk::gdk::{Key, ModifierType};

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/new_article.ui")]
    pub struct NewArticle {
        #[template_child]
        pub article_url_entry: TemplateChild<gtk::Entry>,

        pub sender: OnceCell<Sender<Action>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NewArticle {
        const NAME: &'static str = "NewArticle";
        type Type = super::NewArticle;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();

            klass.install_action("win.accept", None, move |window, _, _| {
                let imp = window.imp();
                let url = Url::parse(&imp.article_url_entry.text()).unwrap();
                let sender = imp.sender.get().unwrap();
                sender.send_blocking(Action::SaveArticle(url)).unwrap();
                window.close();
            });

            klass.add_binding_action(Key::Escape, ModifierType::empty(), "window.close", None)
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NewArticle {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for NewArticle {}

    impl WindowImpl for NewArticle {}

    impl AdwWindowImpl for NewArticle {}
}

glib::wrapper! {
    pub struct NewArticle(ObjectSubclass<imp::NewArticle>)
    @extends gtk::Window, gtk::Widget;
}

#[gtk::template_callbacks]
impl NewArticle {
    pub fn new(sender: Sender<Action>) -> Self {
        let window: Self = Object::new();
        window.init(sender);
        window
    }

    pub fn init(&self, sender: Sender<Action>) {
        let imp = self.imp();
        imp.sender.set(sender).unwrap();
        self.action_set_enabled("win.accept", false);

        let ctx = glib::MainContext::default();
        ctx.spawn_local(glib::clone!(@strong self as widget =>  async move {
            let clipboard_content = widget.clipboard().read_text_future().await;
            if let Ok(Some(text)) = clipboard_content {
                if let Ok(url) = Url::parse(&text) {
                    let entry = &widget.imp().article_url_entry;
                    entry.set_text(url.as_str());
                }
            }
        }));
    }

    #[template_callback]
    fn on_article_url_changed(&self, entry: &gtk::Entry) {
        let url: Result<Url, url::ParseError> = Url::parse(&entry.text());
        if url.is_err() {
            entry.add_css_class("error");
        } else {
            entry.remove_css_class("error");
        }
        self.action_set_enabled("win.accept", url.is_ok());
    }
}
