use super::preview::ArticlePreviewImage;
use crate::models::Article;

use crate::models::ArticleAction;
use glib::Sender;
use gtk::prelude::*;
use std::rc::Rc;

enum ArticleRowAction {
    ImageDownloaded,
}

pub struct ArticleRow {
    pub widget: gtk::ListBoxRow,
    builder: gtk::Builder,
    article: Article,
    preview_image: Rc<ArticlePreviewImage>,
    sender: Sender<ArticleAction>,
}

impl ArticleRow {
    pub fn new(article: Article, sender: Sender<ArticleAction>) -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/article_row.ui");
        get_widget!(builder, gtk::ListBoxRow, article_row);
        let preview_image = ArticlePreviewImage::new();

        let row = Rc::new(Self {
            widget: article_row,
            builder,
            article,
            sender,
            preview_image,
        });

        row.init(row.clone());
        row
    }

    fn do_action(&self, action: ArticleRowAction) -> glib::Continue {
        match action {
            ArticleRowAction::ImageDownloaded => {
                match self.article.get_preview_pixbuf() {
                    Some(pixbuf) => self.preview_image.set_pixbuf(pixbuf),
                    _ => self.preview_image.widget.hide(),
                };
            }
        }
        glib::Continue(true)
    }

    fn init(&self, row: Rc<Self>) {
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
            title_label.set_text(&title);
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

        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let article = self.article.clone();
        spawn!(async move {
            if let Err(_) = article.download_preview_image().await {
                warn!("Failed to download preview image of {:#?} with ID={}", article.title, article.id);
            }
            send!(sender, ArticleRowAction::ImageDownloaded);
        });
        receiver.attach(None, move |action| row.do_action(action));
    }
}
