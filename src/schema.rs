table! {
    articles (id) {
        id -> Integer,
        title -> Text,
        is_archived -> Bool,
        is_public -> Bool,
        is_starred -> Bool,
        mimetype -> Text,
        language -> Text,
        preview_picture -> Text,
        content -> Text,
    }
}
