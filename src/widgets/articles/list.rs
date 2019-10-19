use glib::Sender;
use gtk::prelude::*;
use libhandy::prelude::*;

use crate::application::Action;

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

    pub fn bind_model<F>(&self, model: &gio::ListStore, callback: F)
    where
        for<'r> F: std::ops::Fn(&'r glib::Object) -> gtk::Widget + 'static,
    {
        let listbox: gtk::ListBox = self.builder.get_object("articles_listbox").expect("Failed to retrieve articles_listbox");
        listbox.bind_model(Some(model), callback);
    }
}
