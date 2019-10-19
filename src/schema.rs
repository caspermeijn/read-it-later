table! {
    articles (id) {
        id -> Integer,
        title -> Nullable<Text>,
        is_archived -> Bool,
        is_public -> Bool,
        is_starred -> Bool,
        mimetype -> Nullable<Text>,
        language -> Nullable<Text>,
        preview_picture -> Nullable<Text>,
        content -> Nullable<Text>,
    }
}
