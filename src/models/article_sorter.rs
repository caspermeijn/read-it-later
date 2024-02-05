// Copyright 2024 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use gtk::glib::Cast;

use super::{Article, ArticleObject};

#[derive(Clone, Debug, Default)]
pub struct ArticleSorter {}

impl ArticleSorter {
    pub fn compare_published_at(a: &Article, b: &Article) -> std::cmp::Ordering {
        b.published_at.cmp(&a.published_at)
    }
}

impl From<ArticleSorter> for gtk::Sorter {
    fn from(_sorter: ArticleSorter) -> Self {
        gtk::CustomSorter::new(move |a, b| {
            let article_a = a.downcast_ref::<ArticleObject>().unwrap().article();
            let article_b = b.downcast_ref::<ArticleObject>().unwrap().article();
            ArticleSorter::compare_published_at(article_a, article_b).into()
        })
        .into()
    }
}
