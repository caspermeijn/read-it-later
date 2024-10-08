// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{glib, prelude::*, subclass::prelude::*};

mod imp {
    use std::{cell::RefCell, str::FromStr, sync::OnceLock};

    use gtk::{
        gdk::Texture,
        glib::{clone, subclass::InitializingObject, ParamSpec, Value},
    };
    use url::Url;

    use super::*;
    use crate::models::PreviewImage;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/article_preview.ui")]
    pub struct ArticlePreview {
        #[template_child]
        pub spinner: TemplateChild<gtk::Spinner>,
        #[template_child]
        pub image: TemplateChild<gtk::Picture>,

        pub url: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticlePreview {
        const NAME: &'static str = "ArticlePreview";
        type Type = super::ArticlePreview;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ArticlePreview {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: OnceLock<Vec<ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| vec![glib::ParamSpecString::builder("url").build()])
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "url" => self.url.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "url" => {
                    let url = value.get().unwrap();
                    self.url.replace(url);

                    let ctx = glib::MainContext::default();
                    ctx.spawn_local(clone!(
                        #[weak(rename_to = article_preview)]
                        self,
                        async move {
                            match article_preview.get_preview_picture().await {
                                Some(texture) => article_preview.set_texture(&texture),
                                _ => article_preview.obj().set_visible(false),
                            };
                        }
                    ));
                }
                _ => unimplemented!(),
            }
        }

        fn dispose(&self) {
            self.dispose_template();
        }
    }

    impl WidgetImpl for ArticlePreview {}

    impl ArticlePreview {
        pub async fn get_preview_picture(&self) -> Option<Texture> {
            let url = self.url.borrow().clone();
            match url {
                Some(preview_picture) => {
                    let preview_image = PreviewImage::new(Url::from_str(&preview_picture).ok()?);
                    if !preview_image.exists() {
                        preview_image.download().await.ok()?;
                    }

                    Texture::from_filename(&preview_image.cache).ok()
                }
                None => None,
            }
        }

        pub fn set_texture(&self, texture: &Texture) {
            self.image.set_paintable(Some(texture));
            self.image.set_visible(true);
            self.spinner.set_visible(false);
        }
    }
}

glib::wrapper! {
    pub struct ArticlePreview(ObjectSubclass<imp::ArticlePreview>)
        @extends gtk::Widget;
}
