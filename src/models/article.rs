use anyhow::Result;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use glib::Cast;
use gtk::glib;
use sanitize_html::sanitize_str;
use wallabag_api::types::{Entry, PatchEntry};

use crate::{database, models::ArticleObject, schema::articles};

#[derive(Insertable, Queryable, Eq, PartialEq, Debug, Clone)]
#[diesel(table_name = articles)]
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
    pub fn compare(a: &glib::Object, b: &glib::Object) -> std::cmp::Ordering {
        let article_a = a.downcast_ref::<ArticleObject>().unwrap().article();
        let article_b = b.downcast_ref::<ArticleObject>().unwrap().article();

        article_b.published_at.cmp(&article_a.published_at)
    }

    pub fn load() -> Result<Vec<Self>> {
        use crate::schema::articles::dsl::*;
        let db = database::connection();

        let mut conn = db.get()?;

        articles
            .order(published_at.asc())
            .get_results::<Article>(&mut conn)
            .map_err(From::from)
    }

    pub fn from(entry: Entry) -> Self {
        let published_by = entry.published_by.map(|authors| {
            authors
                .iter()
                .filter_map(|author| author.clone())
                .collect::<Vec<String>>()
                .join(", ")
        });
        let published_at = entry.published_at.map(|datetime| datetime.naive_utc());

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
            reading_time: entry.reading_time as i32,
            base_url: entry.domain_name.clone(),
            url: entry.url,
        }
    }

    pub fn get_patch(&self) -> PatchEntry {
        PatchEntry {
            starred: Some(self.is_starred),
            archive: Some(self.is_archived),
            ..Default::default()
        }
    }

    pub fn get_article_info(&self, display_date: bool) -> String {
        let mut article_info = String::from("");
        if let Some(base_url) = &self.base_url {
            article_info.push_str(base_url);
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
        article_info
    }

    pub fn get_reading_time(&self) -> Option<String> {
        if self.reading_time != 0 {
            return Some(format!("{} minutes", self.reading_time));
        }
        None
    }

    pub fn insert(&self) -> Result<()> {
        let db = database::connection();
        let mut conn = db.get()?;

        diesel::insert_into(articles::table)
            .values(self)
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        let db = database::connection();
        let mut conn = db.get()?;
        use crate::schema::articles::dsl::*;

        diesel::delete(articles.filter(id.eq(&self.id))).execute(&mut conn)?;

        Ok(())
    }

    pub fn toggle_favorite(&mut self) -> Result<()> {
        let db = database::connection();
        let mut conn = db.get()?;
        use crate::schema::articles::dsl::*;

        let target = articles.filter(id.eq(&self.id));
        diesel::update(target)
            .set(is_starred.eq(!self.is_starred))
            .execute(&mut conn)?;

        self.is_starred = !self.is_starred;
        Ok(())
    }

    pub fn toggle_archive(&mut self) -> Result<()> {
        let db = database::connection();
        let mut conn = db.get()?;
        use crate::schema::articles::dsl::*;

        let target = articles.filter(id.eq(&self.id));
        diesel::update(target)
            .set(is_archived.eq(!self.is_archived))
            .execute(&mut conn)?;

        self.is_archived = !self.is_archived;
        Ok(())
    }

    pub fn get_preview(&self) -> Option<String> {
        match &self.content {
            Some(content) => {
                // Regex to remove duplicate spaces
                let re = regex::Regex::new(r"\s+").ok()?;

                let rules = sanitize_html::rules::Rules::new()
                    .delete("br")
                    .delete("img")
                    .delete("figcaption")
                    .allow_comments(false);

                let preview = sanitize_str(&rules, content).ok()?.trim().to_string();
                let mut preview_content = Vec::new();
                let mut counter = 0;
                for line in preview.lines() {
                    if line.len() > 50 {
                        // Ignore small lines
                        preview_content.push(line);
                        counter += 1;
                    }
                    if counter == 1 {
                        // Two lines length for the preview
                        break;
                    }
                }
                let preview = re.replace_all(&preview_content.concat(), " ").to_string(); // Remove duplicate space

                Some(preview)
            }
            None => None,
        }
    }
}
