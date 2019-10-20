use super::preview::{ArticlePreviewImage, ArticlePreviewImageSize};
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
        let preview_image = Rc::new(ArticlePreviewImage::new(ArticlePreviewImageSize::Small));

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
        self.preview_image.widget.set_property_margin(12);
        let preview_image = self.preview_image.clone();
        self.widget.connect_size_allocate(move |_, allocation| {
            if allocation.width > 800 {
                preview_image.widget.show();
                preview_image.set_size(ArticlePreviewImageSize::Big);
            } else if allocation.width > 600 {
                preview_image.widget.show();
                preview_image.set_size(ArticlePreviewImageSize::Small);
            } else if allocation.width > 450 {
                preview_image.widget.show();
                preview_image.set_size(ArticlePreviewImageSize::Mini);
            } else {
                preview_image.widget.hide();
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

        get_widget!(self.builder, gtk::Box, article_container);
        article_container.pack_start(&self.preview_image.widget, false, false, 0);
        if let Some(pixbuf) = self.article.get_preview_pixbuf() {
            self.preview_image.set_pixbuf(pixbuf);
        } else {
            self.preview_image.widget.set_no_show_all(false);
            self.preview_image.widget.hide();
        }
    }
}
