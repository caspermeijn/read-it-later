use super::preview::{ArticlePreviewImage, PreviewImageSize};
use crate::application::Action;
use crate::models::Article;
use glib::Sender;
use gtk::prelude::*;
use std::rc::Rc;

pub struct ArticleRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    sender: Sender<Action>,
    article: Article,
    preview_image: Rc<ArticlePreviewImage>,
}

impl ArticleRow {
    pub fn new(article: Article, sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article_row.ui");
        let widget: gtk::ListBoxRow = builder.get_object("article_row").expect("Failed to retrieve article_row");
        let preview_image = ArticlePreviewImage::new(PreviewImageSize::Small);

        let row = Self {
            widget,
            builder,
            sender,
            article,
            preview_image,
        };

        row.init();
        row
    }

    pub fn set_on_click_callback<F>(&self, callback: F)
    where
        for<'r, 's> F: std::ops::Fn(&'r gtk::EventBox, &'s gdk::EventButton) -> gtk::Inhibit + 'static,
    {
        get_widget!(self.builder, gtk::EventBox, event_box);
        event_box.connect_button_press_event(callback);
    }

    fn init(&self) {
        get_widget!(self.builder, gtk::Box, article_container);
        get_widget!(self.builder, gtk::Box, content_box);
        article_container.pack_start(&self.preview_image.widget, false, false, 0);
        if let Some(pixbuf) = self.article.get_preview_pixbuf() {
            let preview_image = self.preview_image.clone();
            self.widget.connect_size_allocate(move |_, allocation| {
                if allocation.width <= 450 {
                    preview_image.set_size(PreviewImageSize::Small);
                } else {
                    preview_image.set_size(PreviewImageSize::Big);
                }
            });
            self.preview_image.set_pixbuf(pixbuf);
        } else {
            self.preview_image.widget.set_no_show_all(false);
            self.preview_image.widget.hide();
        }

        get_widget!(self.builder, gtk::Label, title_label);
        if let Some(title) = &self.article.title {
            title_label.set_text(&title);
        }

        get_widget!(self.builder, gtk::Label, info_label);
        let mut article_info = String::from("");
        if let Some(base_url) = &self.article.base_url {
            article_info.push_str(&format!("{} | ", base_url));
        }
        if let Some(authors) = &self.article.published_by {
            article_info.push_str(&format!("by {} ", authors));
        }
        if &article_info != "" {
            info_label.set_text(&article_info);
        } else {
            info_label.set_no_show_all(false);
            info_label.hide();
        }

        get_widget!(self.builder, gtk::Label, content_label);
        if let Ok(Some(preview)) = self.article.get_preview() {
            content_label.set_markup(&preview);
        }
    }
}
