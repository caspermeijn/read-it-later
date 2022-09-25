use crate::models::Article;
use crate::models::ArticleAction;
use crate::widgets::articles::preview::ArticlePreview;
use gtk::glib;
use gtk::glib::clone;
use gtk::glib::Sender;
use gtk::prelude::*;
use gtk_macros::{get_widget, send, spawn};
use log::error;
use std::rc::Rc;

pub struct ArticleRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    article: Article,
    preview_image: ArticlePreview,
    sender: Sender<ArticleAction>,
    client: Rc<isahc::HttpClient>,
}

impl ArticleRow {
    pub fn new(article: Article, client: Rc<isahc::HttpClient>, sender: Sender<ArticleAction>) -> Self {
        ArticlePreview::ensure_type();

        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/article_row.ui");
        get_widget!(builder, gtk::ListBoxRow, article_row);
        get_widget!(builder, ArticlePreview, preview_image);

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
        get_widget!(self.builder, gtk::ListBoxRow, article_row);

        let event_controller = gtk::GestureClick::new();
        article_row.add_controller(&event_controller);
        event_controller.connect_pressed(
            clone!(@strong self.sender as sender, @strong self.article as article => move |_, _, _, _| {
                send!(sender, ArticleAction::Open(article.clone()));
            }),
        );

        get_widget!(self.builder, gtk::Label, title_label);
        if let Some(title) = &self.article.title {
            title_label.set_text(title);
        }

        get_widget!(self.builder, gtk::Label, info_label);
        match self.article.get_article_info(false) {
            Some(article_info) => info_label.set_text(&article_info),
            None => {
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
                Ok(Some(pixbuf)) => preview_image.set_pixbuf(&pixbuf),
                _ => preview_image.hide(),
            };
        });
    }
}
