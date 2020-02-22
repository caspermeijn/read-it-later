use glib::object::Cast;
use glib::Sender;

use std::rc::Rc;

use crate::application::Action;
use crate::widgets::LoginWidget;

pub struct LoginView {
    pub widget: Rc<LoginWidget>,
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
        self.widget.on_login_clicked(move |_| {
            if let Some(client_config) = login_widget.get_wallabag_client_config() {
                send!(sender, Action::SetClientConfig(client_config));
            }
        });
    }
}
