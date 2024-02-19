// Copyright 2022 Bilal Elmoussaoui <belmouss@redhat.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::{glib, prelude::*, subclass::prelude::*};

use crate::models::Article;

mod imp {
    use std::{cell::OnceCell, sync::OnceLock};

    use super::*;

    #[derive(Default)]
    pub struct ArticleObject {
        pub article: OnceCell<Article>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ArticleObject {
        const NAME: &'static str = "ArticleObject";
        type Type = super::ArticleObject;
    }

    impl ObjectImpl for ArticleObject {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecString::builder("title").read_only().build(),
                    glib::ParamSpecString::builder("preview-text")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("description")
                        .read_only()
                        .build(),
                    glib::ParamSpecString::builder("cover-picture-url")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            let article = self.article.get().unwrap();
            match pspec.name() {
                "title" => article.title.clone().to_value(),
                "preview-text" => article.get_preview().to_value(),
                "description" => article.get_article_info(false).to_value(),
                "cover-picture-url" => article.preview_picture.clone().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct ArticleObject(ObjectSubclass<imp::ArticleObject>);
}

impl ArticleObject {
    pub fn new(article: Article) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.imp().article.set(article).unwrap();
        obj
    }

    pub fn article(&self) -> &Article {
        self.imp().article.get().unwrap()
    }
}
