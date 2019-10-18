use crate::database;
use crate::database::Insert;
use crate::schema::articles;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use failure::Error;
use wallabag_api::types::Entry;

#[derive(Insertable, Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[table_name = "articles"]
pub struct Article {
    id: i32,
    title: Option<String>,
    is_archived: bool,
    is_public: bool,
    is_starred: bool,
    mimetype: Option<String>,
    language: Option<String>,
    preview_picture: Option<String>,
    content: Option<String>,
}

impl Article {
    pub fn from(entry: Entry) -> Self {
        Article {
            id: (entry.id.as_int()) as i32,
            title: entry.title.clone(),
            is_archived: entry.is_archived,
            is_public: entry.is_public,
            is_starred: entry.is_starred,
            mimetype: entry.mimetype.clone(),
            language: entry.language.clone(),
            preview_picture: entry.preview_picture.clone(),
            content: entry.content.clone(),
        }
    }

    pub fn insert(&self) -> Result<(), database::Error> {
        let db = database::connection();
        let conn = db.get()?;

        println!("{:#?}", self.title);
        diesel::insert_into(articles::table).values(self).execute(&conn)?;

        Ok(())
    }
}
