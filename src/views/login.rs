use gtk::glib::Sender;
use gtk_macros::send;
use log::error;

use crate::application::Action;

pub struct LoginView {
    pub widget: crate::widgets::Login,
    pub name: String,
    sender: Sender<Action>,
}

impl LoginView {
    pub fn new(sender: Sender<Action>) -> Self {
        let widget = crate::widgets::Login::new();

        let view = Self {
            widget,
            name: "login".to_string(),
            sender,
        };
        view.init();
        view
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
