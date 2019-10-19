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
        published_by -> Nullable<Text>,
        published_at -> Nullable<Timestamp>,
        reading_time -> Integer,
        base_url -> Nullable<Text>,
    }
}
