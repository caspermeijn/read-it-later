// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use async_std::channel::Sender;
use gio::prelude::*;
use glib::clone;
use gtk::{gio, glib, subclass::prelude::*};

use crate::models::{ArticleAction, ArticleObject};

mod imp {
    use std::{cell::OnceCell, sync::OnceLock};

    use glib::{subclass::InitializingObject, ParamSpec, ParamSpecString, Value};
    use gtk::prelude::*;

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/articles_list.ui")]
    pub struct ArticlesListWidget {
        #[template_child]
        pub empty_status: TemplateChild<adw::StatusPage>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub selection_model: TemplateChild<gtk::NoSelection>,
        pub sender: OnceCell<Sender<ArticleAction>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for ArticlesListWidget {
        const NAME: &'static str = "ArticlesListWidget";
        type Type = super::ArticlesListWidget;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticlesListWidget {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES
                .get_or_init(|| vec![ParamSpecString::builder("placeholder-icon-name").build()])
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "placeholder-icon-name" => self.empty_status.icon_name().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "placeholder-icon-name" => {
                    let icon_name = value.get().unwrap();
                    self.empty_status.set_icon_name(icon_name)
                }
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticlesListWidget {}

    #[gtk::template_callbacks]
    impl ArticlesListWidget {
        #[template_callback]
        async fn handle_row_activate(&self, position: u32, list_view: gtk::ListView) {
            let item = list_view.model().unwrap().item(position).unwrap();
            let article = item.downcast_ref::<ArticleObject>().unwrap().article();
            let sender = self.sender.get().unwrap();
            sender
                .send(ArticleAction::Open(article.clone()))
                .await
                .unwrap();
        }
    }
}

glib::wrapper! {
    pub struct ArticlesListWidget(ObjectSubclass<imp::ArticlesListWidget>)
        @extends gtk::Widget;
}

impl ArticlesListWidget {
    fn update_model_empty(&self, model: &impl IsA<gio::ListModel>) {
        if model.n_items() == 0 {
            self.imp().stack.set_visible_child_name("empty")
        } else {
            self.imp().stack.set_visible_child_name("list")
        }
    }

    pub fn bind_model(&self, model: &impl IsA<gio::ListModel>) {
        self.update_model_empty(model);
        model.connect_items_changed(clone!(
            #[strong(rename_to = list_widget)]
            self,
            move |model, _, _, _| {
                list_widget.update_model_empty(model);
            }
        ));

        self.imp().selection_model.set_model(Some(model));
    }

    pub fn set_sender(&self, sender: Sender<ArticleAction>) {
        self.imp().sender.set(sender).unwrap();
    }
}
