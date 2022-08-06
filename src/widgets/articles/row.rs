use super::preview::ArticlePreviewImage;
use crate::models::Article;

use crate::models::ArticleAction;
use glib::Sender;
use gtk::prelude::*;
use std::rc::Rc;

pub struct ArticleRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    article: Article,
    preview_image: Rc<ArticlePreviewImage>,
    sender: Sender<ArticleAction>,
    client: Rc<isahc::HttpClient>,
}

impl ArticleRow {
    pub fn new(article: Article, client: Rc<isahc::HttpClient>, sender: Sender<ArticleAction>) -> Self {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/article_row.ui");
        get_widget!(builder, gtk::ListBoxRow, article_row);
        let preview_image = ArticlePreviewImage::new();

        let row = Self {
            widget: article_row,
            builder,
            article,
            sender,
            preview_image,
            client,
        };

        row.init();
        row
    }

    fn init(&self) {
        get_widget!(self.builder, gtk::EventBox, event_box);
        event_box.connect_button_press_event(
            clone!(@strong self.sender as sender, @strong self.article as article => move |_, _| {
                send!(sender, ArticleAction::Open(article.clone()));
                gtk::Inhibit(false)
            }),
        );
        get_widget!(self.builder, gtk::Box, article_container);
        article_container.pack_end(&self.preview_image.widget, false, false, 0);

        get_widget!(self.builder, gtk::Label, title_label);
        if let Some(title) = &self.article.title {
            title_label.set_text(title);
        }

        get_widget!(self.builder, gtk::Label, info_label);
        match self.article.get_article_info(false) {
            Some(article_info) => info_label.set_text(&article_info),
            None => {
                info_label.set_no_show_all(false);
                info_label.hide();
            }
        };

        get_widget!(self.builder, gtk::Label, content_label);
        if let Ok(Some(preview)) = self.article.get_preview() {
            content_label.set_text(&preview);
        }

        let article = self.article.clone();
        let preview_image = self.preview_image.clone();
        let client = self.client.clone();
        spawn!(async move {
            match article.get_preview_picture(client).await {
                Ok(Some(pixbuf)) => preview_image.set_pixbuf(pixbuf),
                _ => preview_image.widget.hide(),
            };
        });
    }
}
