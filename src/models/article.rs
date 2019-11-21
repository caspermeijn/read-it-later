use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use gdk_pixbuf::Pixbuf;
use sanitize_html::sanitize_str;
use wallabag_api::types::{Entry, PatchEntry};

use crate::database;
use crate::models::{ArticlesFilter, PreviewImage};
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
    pub url: Option<String>,
}

impl Article {
    pub fn load(filter: &ArticlesFilter) -> Result<Vec<Self>> {
        use crate::schema::articles::dsl::*;
        let db = database::connection();

        let conn = db.get()?;

        let mut query = articles.order(published_at.asc()).into_boxed();

        if let Some(starred) = &filter.starred {
            query = query.filter(is_starred.eq(starred));
        }
        if let Some(archived) = &filter.archived {
            query = query.filter(is_archived.eq(archived));
        }
        query.get_results::<Article>(&conn).map_err(From::from)
    }

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
            url: entry.url.clone(),
        }
    }

    pub fn get_patch(&self) -> PatchEntry {
        let mut patch = PatchEntry::default();
        patch.starred = Some(self.is_starred);
        patch.archive = Some(self.is_archived);
        patch
    }

    pub fn get_article_info(&self, display_date: bool) -> Option<String> {
        let mut article_info = String::from("");
        if let Some(base_url) = &self.base_url {
            article_info.push_str(&format!("{}", base_url));
        }
        if let Some(authors) = &self.published_by {
            article_info.push_str(&format!(" | by {} ", authors));
        }

        if display_date {
            if let Some(published_date) = &self.published_at {
                let formatted_date = published_date.format("%d %b %Y").to_string();
                article_info.push_str(&format!(" | on {} ", formatted_date));
            }
        }

        if let Some(reading_time) = self.get_reading_time() {
            article_info.push_str(&format!(" | {} ", reading_time));
        }
        Some(article_info)
    }

    pub fn get_reading_time(&self) -> Option<String> {
        if self.reading_time != 0 {
            return Some(format!("{} minutes", self.reading_time));
        }
        None
    }

    pub fn insert(&self) -> Result<()> {
        let db = database::connection();
        let conn = db.get()?;

        diesel::insert_into(articles::table).values(self).execute(&conn)?;

        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        let db = database::connection();
        let conn = db.get()?;
        use crate::schema::articles::dsl::*;

        diesel::delete(articles.filter(id.eq(&self.id))).execute(&conn)?;

        Ok(())
    }

    pub fn toggle_favorite(&mut self) -> Result<()> {
        let db = database::connection();
        let conn = db.get()?;
        use crate::schema::articles::dsl::*;

        let target = articles.filter(id.eq(&self.id));
        diesel::update(target).set(is_starred.eq(!self.is_starred)).execute(&conn)?;

        self.is_starred = !self.is_starred;
        Ok(())
    }

    pub fn toggle_archive(&mut self) -> Result<()> {
        let db = database::connection();
        let conn = db.get()?;
        use crate::schema::articles::dsl::*;

        let target = articles.filter(id.eq(&self.id));
        diesel::update(target).set(is_archived.eq(!self.is_archived)).execute(&conn)?;

        self.is_archived = !self.is_archived;
        Ok(())
    }

    pub async fn download_preview_image(&self) -> Result<()> {
        if let Some(preview_picture) = &self.preview_picture {
            let preview_image = PreviewImage::new(preview_picture.to_string());
            preview_image.download().await?;
        }
        Ok(())
    }

    pub fn get_preview_pixbuf(&self) -> Option<Pixbuf> {
        if let Some(preview_picture) = &self.preview_picture {
            let cache_path = PreviewImage::get_cache_of(&preview_picture);

            return match gdk_pixbuf::Pixbuf::new_from_file(cache_path) {
                Ok(pixbuf) => Some(pixbuf),
                Err(_) => None,
            };
        }
        None
    }

    pub fn get_preview(&self) -> Result<Option<String>> {
        match &self.content {
            Some(content) => {
                // Regex to remove duplicate spaces
                let re = regex::Regex::new(r"\s+")?;

                let rules = sanitize_html::rules::Rules::new()
                    .delete("br")
                    .delete("img")
                    .delete("figcaption")
                    .allow_comments(false);

                let preview = sanitize_str(&rules, &content)?.trim().to_string();
                let mut preview_content = Vec::new();
                let mut counter = 0;
                for line in preview.lines() {
                    if line.len() > 50 {
                        // Ignore small lines
                        preview_content.push(line);
                        counter = counter + 1;
                    }
                    if counter == 1 {
                        // Two lines length for the preview
                        break;
                    }
                }
                let preview = re.replace_all(&preview_content.concat(), " ").to_string(); // Remove duplicate space

                Ok(Some(preview))
            }
            None => Ok(None),
        }
    }
}
