use super::preview::{ArticlePreviewImage, ArticlePreviewImageType};
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
        let preview_image = ArticlePreviewImage::new(ArticlePreviewImageType::Thumbnail);

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
            self.preview_image.set_pixbuf(pixbuf);
        } else {
            self.preview_image.widget.set_no_show_all(false);
            self.preview_image.widget.hide();
        }
        let preview_image = self.preview_image.clone();
        self.widget.connect_size_allocate(move |_, allocation| {
            if allocation.width > 700 {
                article_container.set_orientation(gtk::Orientation::Horizontal);
                article_container.reorder_child(&preview_image.widget, 1);
                preview_image.set_image_type(ArticlePreviewImageType::Thumbnail);
                // The size of thumbnail
                let mut height = 200;
                if allocation.height < 200 {
                    height = allocation.height;
                }
                preview_image.set_size(200, height);
                content_box.set_property_margin(12);
            } else {
                article_container.set_orientation(gtk::Orientation::Vertical);
                article_container.reorder_child(&preview_image.widget, 0);
                preview_image.set_image_type(ArticlePreviewImageType::Cover);
                let mut width = 340;
                if allocation.width > 360 {
                    width = allocation.width - 25;
                }

                preview_image.set_size(width, 200);
                content_box.set_property_margin(0);
            }
        });

        get_widget!(self.builder, gtk::Label, title_label);
        if let Some(title) = &self.article.title {
            title_label.set_text(&title);
        }

        get_widget!(self.builder, gtk::Label, info_label);
        if &self.article.get_info() != "" {
            info_label.set_text(&self.article.get_info());
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
