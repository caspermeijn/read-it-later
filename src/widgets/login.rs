use gtk::prelude::*;
use std::rc::Rc;
use wallabag_api::types::Config;

pub struct LoginWidget {
    pub widget: gtk::ScrolledWindow,
    builder: gtk::Builder,
}

impl LoginWidget {
    pub fn new() -> Rc<Self> {
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/login.ui");
        let widget: gtk::ScrolledWindow = builder.get_object("login").expect("Failed to retrieve LoginWidget");

        let login_widget = Rc::new(Self { widget, builder });

        login_widget
    }

    pub fn get_wallabag_client_config(&self) -> Option<Config> {
        let instance_entry: gtk::Entry = self.builder.get_object("instance_entry").expect("Failed to retrieve instance_entry");
        let client_id_entry: gtk::Entry = self.builder.get_object("client_id_entry").expect("Failed to retrieve client_id_entry");
        let client_secret_entry: gtk::Entry = self.builder.get_object("client_secret_entry").expect("Failed to retrieve client_secret_entry");
        let username_entry: gtk::Entry = self.builder.get_object("username_entry").expect("Failed to retrieve username_entry");
        let password_entry: gtk::Entry = self.builder.get_object("password_entry").expect("Failed to retrieve password_entry");

        Some(Config {
            client_id: client_id_entry.get_text()?.to_string(),
            client_secret: client_secret_entry.get_text()?.to_string(),
            username: username_entry.get_text()?.to_string(),
            password: password_entry.get_text()?.to_string(),
            base_url: instance_entry.get_text()?.to_string(),
        })
    }

    pub fn on_login_clicked<F>(&self, callback: F)
    where
        for<'r> F: std::ops::Fn(&'r gtk::Button) + 'static,
    {
        let login_button: gtk::Button = self.builder.get_object("login_button").expect("Failed to retrieve login_button");

        login_button.connect_clicked(callback);
    }
}
