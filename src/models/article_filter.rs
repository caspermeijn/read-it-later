use glib::Cast;
use gtk::glib;

use crate::models::ArticleObject;

#[derive(Clone, Debug, Default)]
pub struct ArticlesFilter {
    pub archived: Option<bool>,
    pub starred: Option<bool>,
}

impl ArticlesFilter {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn favorites() -> Self {
        ArticlesFilter {
            starred: Some(true),
            ..Default::default()
        }
    }

    pub fn archive() -> Self {
        ArticlesFilter {
            archived: Some(true),
            ..Default::default()
        }
    }

    pub fn unread() -> Self {
        ArticlesFilter {
            archived: Some(false),
            ..Default::default()
        }
    }
}

impl From<ArticlesFilter> for gtk::Filter {
    fn from(filter: ArticlesFilter) -> Self {
        gtk::CustomFilter::new(move |obj| {
            let article = obj.downcast_ref::<ArticleObject>().unwrap().article();
            if let Some(filter_archived) = filter.archived {
                article.is_archived == filter_archived
            } else if let Some(filter_starred) = filter.starred {
                article.is_starred == filter_starred
            } else {
                unimplemented!()
            }
        })
        .into()
    }
}
