use gio::prelude::*;
use glib::Sender;
use gtk::prelude::*;

use super::row::ArticleRow;
use crate::models::{Article, ArticleAction, ObjectWrapper};

pub struct ArticlesListWidget {
    pub widget: libhandy::Column,
    builder: gtk::Builder,
    sender: Sender<ArticleAction>,
}

impl ArticlesListWidget {
    pub fn new(sender: Sender<ArticleAction>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/articles_list.ui");
        get_widget!(builder, libhandy::Column, articles_list);

        let list_widget = Self {
            builder,
            widget: articles_list,
            sender,
        };

        list_widget
    }

    pub fn bind_model(&self, model: &gio::ListStore, icon: &str, empty_msg: &str) {
        get_widget!(self.builder, gtk::Label, empty_label);
        get_widget!(self.builder, gtk::Image, empty_image);

        empty_label.set_text(empty_msg);
        empty_image.set_from_icon_name(Some(icon), gtk::IconSize::Dialog);

        get_widget!(self.builder, gtk::Stack, stack);
        if model.get_n_items() == 0 {
            stack.set_visible_child_name("empty");
        } else {
            stack.set_visible_child_name("articles");
        }
        model.connect_items_changed(move |model, _, _, _| {
            if model.get_n_items() == 0 {
                stack.set_visible_child_name("empty");
            } else {
                stack.set_visible_child_name("articles");
            }
        });
        get_widget!(self.builder, gtk::ListBox, articles_listbox);
        articles_listbox.bind_model(
            Some(model),
            clone!(@strong self.sender as sender => move |article| {
                let article: Article = article.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
                let row = ArticleRow::new(article.clone());
                row.set_on_click_callback(clone!(@strong sender => move |_, _| {
                    send!(sender, ArticleAction::Open(article.clone()));
                    gtk::Inhibit(false)
                }));
                let widget = row.widget.clone();
                widget.upcast::<gtk::Widget>()
            }),
        );
    }
}
