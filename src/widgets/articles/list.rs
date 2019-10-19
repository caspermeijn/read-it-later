use glib::Sender;
use gtk::prelude::*;
use libhandy::prelude::*;

use super::row::ArticleRow;
use crate::application::Action;
use crate::models::{Article, ObjectWrapper};

pub struct ArticlesListWidget {
    pub widget: libhandy::Column,
    builder: gtk::Builder,
    sender: Sender<Action>,
}

impl ArticlesListWidget {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/articles_list.ui");
        let widget: libhandy::Column = builder.get_object("articles_list").expect("Couldn't retrieve articles_list");

        let list_widget = Self { builder, widget, sender };

        list_widget.init();
        list_widget
    }

    fn init(&self) {
        let listbox: gtk::ListBox = self.builder.get_object("articles_listbox").expect("Failed to retrieve articles_listbox");
        listbox.set_header_func(Some(Box::new(move |row1, row2| {
            if let Some(_) = row2 {
                let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                row1.set_header(Some(&separator));
                separator.show();
            }
        })));
    }

    pub fn bind_model(&self, model: &gio::ListStore) {
        let listbox: gtk::ListBox = self.builder.get_object("articles_listbox").expect("Failed to retrieve articles_listbox");
        let sender = self.sender.clone();
        listbox.bind_model(Some(model), move |article| {
            let article: Article = article.downcast_ref::<ObjectWrapper>().unwrap().deserialize();
            let row = ArticleRow::new(article.clone(), sender.clone());
            let sender = sender.clone();
            row.set_on_click_callback(move |_, _| {
                sender.send(Action::LoadArticle(article.clone())).unwrap();
                gtk::Inhibit(false)
            });
            row.widget.upcast::<gtk::Widget>()
        });
    }
}
