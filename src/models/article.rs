use diesel::RunQueryDsl;
use failure::Error;
use gdk_pixbuf::Pixbuf;
use html2pango::markup_html;
use sanitize_html::rules::Element;
use sanitize_html::sanitize_str;
use wallabag_api::types::Entry;

use super::preview_image::{PreviewImage, PreviewImageType};
use crate::database;
use crate::schema::articles;

#[derive(Insertable, Queryable, PartialEq, Debug, Clone, Serialize, Deserialize)]
#[table_name = "articles"]
pub struct Article {
    pub id: i32,
    pub title: Option<String>,
    pub is_archived: bool,
    is_public: bool,
    pub is_starred: bool,
    mimetype: Option<String>,
    language: Option<String>,
    pub preview_picture: Option<String>,
    pub content: Option<String>,
    pub published_by: Option<String>,
    pub published_at: Option<chrono::NaiveDateTime>,
    pub reading_time: i32,
    pub base_url: Option<String>,
}

impl Article {
    pub fn from(entry: Entry) -> Self {
        let published_by = match entry.published_by.clone() {
            Some(published_by) => Some(
                published_by
                    .iter()
                    .filter(|author| author.is_some())
                    .map(|author| author.clone().unwrap())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
            None => None,
        };
        let published_at = match entry.published_at.clone() {
            Some(datetime) => Some(datetime.naive_utc()),
            None => None,
        };

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
            published_by,
            published_at,
            reading_time: entry.reading_time.clone() as i32,
            base_url: entry.domain_name.clone(),
        }
    }

    pub fn insert(&self) -> Result<(), database::Error> {
        let db = database::connection();
        let conn = db.get()?;

        println!("{:#?}", self.title);
        diesel::insert_into(articles::table).values(self).execute(&conn)?;

        Ok(())
    }

    pub fn get_preview_pixbuf(&self, img_type: PreviewImageType) -> Option<Pixbuf> {
        if let Some(preview_picture) = &self.preview_picture {
            let preview_image = PreviewImage::new(preview_picture.to_string());
            if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::new_from_file(preview_image.get_cache_path()) {
                let mut dest_width = pixbuf.get_width();
                let mut dest_height = pixbuf.get_height();
                match img_type {
                    PreviewImageType::Small => {
                        if dest_width > 200 {
                            dest_width = 200;
                        }
                        if dest_height > 200 {
                            dest_height = 200;
                        }
                    }
                    PreviewImageType::Large => {
                        if dest_width > 500 {
                            dest_width = 800;
                        }
                        if dest_height > 360 {
                            dest_height = 360;
                        }
                    }
                }
                let scaled_pixbuf = pixbuf.scale_simple(dest_width, dest_height, gdk_pixbuf::InterpType::Bilinear);
                return scaled_pixbuf;
            }
        }
        None
    }

    pub fn get_info(&self) -> String {
        let mut article_info = String::from("");
        if let Some(base_url) = &self.base_url {
            article_info.push_str(&format!("{} | ", base_url));
        }
        if let Some(authors) = &self.published_by {
            article_info.push_str(&format!("by {} ", authors));
        }
        if let Some(published_date) = &self.published_at {
            article_info.push_str(&format!("on {} ", published_date));
        }
        article_info
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
