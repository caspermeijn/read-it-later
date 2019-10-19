use crate::application::Action;
use crate::models::{Article, PreviewImageType};
use glib::Sender;
use gtk::prelude::*;

pub struct ArticleRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    sender: Sender<Action>,
    article: Article,
}

impl ArticleRow {
    pub fn new(article: Article, sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article_row.ui");
        let widget: gtk::ListBoxRow = builder.get_object("article_row").expect("Failed to retrieve article_row");

        let row = Self {
            widget,
            builder,
            sender,
            article,
        };

        row.init();
        row
    }

    pub fn set_on_click_callback<F>(&self, callback: F)
    where
        for<'r, 's> F: std::ops::Fn(&'r gtk::EventBox, &'s gdk::EventButton) -> gtk::Inhibit + 'static,
    {
        let event_box: gtk::EventBox = self.builder.get_object("event_box").expect("Failed to load event_box");
        event_box.connect_button_press_event(callback);
    }

    fn init(&self) {
        let title_label: gtk::Label = self.builder.get_object("title_label").expect("Failed to retrieve title_label");
        if let Some(title) = &self.article.title {
            title_label.set_text(&title);
        }
        let info_label: gtk::Label = self.builder.get_object("info_label").expect("Failed to retrieve info_label");
        if &self.article.get_info() != "" {
            info_label.set_text(&self.article.get_info());
        } else {
            info_label.set_no_show_all(false);
            info_label.hide();
        }

        let content_label: gtk::Label = self.builder.get_object("content_label").expect("Failed to retrieve content_label");
        if let Ok(Some(preview)) = self.article.get_preview() {
            content_label.set_markup(&preview);
        }

        let preview_image: gtk::Image = self.builder.get_object("preview_image").expect("Failed to retrieve preview_image");
        if let Some(pixbuf) = &self.article.get_preview_pixbuf(PreviewImageType::Small) {
            preview_image.set_from_pixbuf(Some(&pixbuf));
        } else {
            preview_image.set_no_show_all(false);
            preview_image.hide();
        }
    }
}
