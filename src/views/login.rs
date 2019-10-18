use glib::object::Cast;
use glib::Sender;
use gtk::prelude::*;

use std::rc::Rc;

use crate::application::Action;
use crate::widgets::LoginWidget;

pub struct LoginView {
    widget: Rc<LoginWidget>,
    pub name: String,
    sender: Sender<Action>,
}

impl LoginView {
    pub fn new(sender: Sender<Action>) -> Self {
        let widget = LoginWidget::new();

        let view = Self {
            widget,
            name: "login".to_string(),
            sender,
        };
        view.init();
        view
    }

    pub fn get_widget(&self) -> gtk::Widget {
        let widget = self.widget.widget.clone();
        widget.upcast::<gtk::Widget>()
    }

    fn init(&self) {
        let sender = self.sender.clone();
        let login_widget = self.widget.clone();
        self.widget.on_login_clicked(move |login_button| {
            login_button.set_sensitive(false);
            if let Some(client_config) = login_widget.get_wallabag_client_config() {
                sender.send(Action::SetClientConfig(client_config)).expect("Failed to set client config");
            }
        });
    }
}
