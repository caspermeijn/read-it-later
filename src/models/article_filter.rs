#[derive(Clone, Default)]
pub struct ArticlesFilter {
    pub archived: Option<bool>,
    pub starred: Option<bool>,
}

impl ArticlesFilter {
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
