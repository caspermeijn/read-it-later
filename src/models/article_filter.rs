#[derive(Clone)]
pub struct ArticlesFilter {
    pub archived: Option<bool>,
    pub starred: Option<bool>,
}

impl ArticlesFilter {
    pub fn favorites() -> Self {
        let mut filter = ArticlesFilter::default();
        filter.starred = Some(true);
        filter
    }

    pub fn archive() -> Self {
        let mut filter = ArticlesFilter::default();
        filter.archived = Some(true);
        filter
    }

    pub fn unread() -> Self {
        let mut filter = ArticlesFilter::default();
        filter.archived = Some(false);
        filter
    }
}

impl Default for ArticlesFilter {
    fn default() -> Self {
        Self {
            archived: None,
            starred: None,
        }
    }
}
