use glib::object::Cast;
use glib::Sender;
use gtk::prelude::*;

use crate::application::Action;

pub struct SyncingView {
    widget: gtk::Box,
    builder: gtk::Builder,
    pub name: String,
    sender: Sender<Action>,
}

impl SyncingView {
    pub fn new(sender: Sender<Action>) -> Self {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/syncing.ui");
        let widget: gtk::Box = builder.get_object("syncing").expect("Failed to retrieve SyncingWidget");

        let view = Self {
            widget,
            name: "syncing".to_string(),
            sender,
            builder,
        };
        view.init();
        view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    fn init(&self) {
        //
    }
}
