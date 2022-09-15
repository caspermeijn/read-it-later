use gtk::gio;
use gtk::gio::prelude::*;
use gtk::glib::clone;
use gtk::glib::Sender;
use gtk::prelude::*;
use gtk_macros::get_widget;
use std::rc::Rc;

use super::row::ArticleRow;
use crate::models::{Article, ArticleAction, ObjectWrapper};

pub struct ArticlesListWidget {
    pub widget: adw::Clamp,
    builder: gtk::Builder,
    sender: Sender<ArticleAction>,
    client: Rc<isahc::HttpClient>,
}

impl ArticlesListWidget {
    pub fn new(sender: Sender<ArticleAction>, client: Rc<isahc::HttpClient>) -> Self {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/articles_list.ui");
        get_widget!(builder, adw::Clamp, articles_list);

        Self {
            builder,
            widget: articles_list,
            sender,
            client,
        }
    }

    pub fn bind_model(&self, model: &gio::ListStore, icon: &str) {
        get_widget!(self.builder, adw::StatusPage, empty_status);
        empty_status.set_icon_name(Some(icon));

        get_widget!(self.builder, gtk::Stack, stack);
        if model.n_items() == 0 {
            stack.set_visible_child_name("empty");
        } else {
            stack.set_visible_child_name("articles");
        }
        model.connect_items_changed(move |model, _, _, _| {
            if model.n_items() == 0 {
                stack.set_visible_child_name("empty");
            } else {
                stack.set_visible_child_name("articles");
            }
        });
        get_widget!(self.builder, gtk::ListBox, articles_listbox);

        articles_listbox.bind_model(
            Some(model),
            clone!(@strong self.sender as sender, @strong self.client as client => move |article| {
                let article: Article = article.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
                let row = ArticleRow::new(article, client.clone(), sender.clone());
                row.widget.upcast::<gtk::Widget>()
            }),
        );
    }
}
