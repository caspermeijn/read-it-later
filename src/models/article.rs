use diesel::RunQueryDsl;
use failure::Error;
use gdk_pixbuf::Pixbuf;
use html2pango::markup_html;
use sanitize_html::rules;
use sanitize_html::rules::Element;
use sanitize_html::sanitize_str;
use wallabag_api::types::Entry;

use super::preview_image::PreviewImage;
use crate::database;
use crate::schema::articles;

#[derive(Insertable, Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[table_name = "articles"]
pub struct Article {
    id: i32,
    pub title: Option<String>,
    is_archived: bool,
    is_public: bool,
    is_starred: bool,
    mimetype: Option<String>,
    language: Option<String>,
    pub preview_picture: Option<String>,
    pub content: Option<String>,
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

    pub fn get_preview_pixbuf(&self) -> Option<Pixbuf> {
        if let Some(preview_picture) = &self.preview_picture {
            println!("{:#?}", preview_picture);
            let preview_image = PreviewImage::new(preview_picture.to_string());
        }
        None
    }

    pub fn get_preview(&self) -> Result<Option<String>, Error> {
        match &self.content {
            Some(content) => {
                let rules = sanitize_html::rules::Rules::new()
                    .allow_comments(false)
                    .element(Element::new("br"))
                    .element(Element::new("a"));

                let mut preview = sanitize_str(&rules, &content)?;
                preview.truncate(300);

                let preview_markup = markup_html(&preview)?;
                Ok(Some(preview_markup))
            }
            None => Ok(None),
        }
    }
}
