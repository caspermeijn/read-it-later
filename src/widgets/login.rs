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
        get_widget!(builder, gtk::ScrolledWindow, login);

        Rc::new(Self { widget: login, builder })
    }

    pub fn get_wallabag_client_config(&self) -> Option<Config> {
        get_widget!(self.builder, gtk::Entry, instance_entry);
        get_widget!(self.builder, gtk::Entry, client_id_entry);
        get_widget!(self.builder, gtk::Entry, client_secret_entry);
        get_widget!(self.builder, gtk::Entry, username_entry);
        get_widget!(self.builder, gtk::Entry, password_entry);

        let instance = instance_entry.get_text()?;
        let instance = instance.trim_end_matches("/").to_string();
        if let Err(err) = url::Url::parse(&instance) {
            error!("The instance url is invalid {}", err);
            instance_entry.get_style_context().add_class("error");
            return None;
        }
        instance_entry.get_style_context().remove_class("error");

        Some(Config {
            client_id: client_id_entry.get_text()?.to_string(),
            client_secret: client_secret_entry.get_text()?.to_string(),
            username: username_entry.get_text()?.to_string(),
            password: password_entry.get_text()?.to_string(),
            base_url: instance,
        })
    }

    pub fn on_login_clicked<F>(&self, callback: F)
    where
        for<'r> F: std::ops::Fn(&'r gtk::Button) + 'static,
    {
        get_widget!(self.builder, gtk::Button, login_button);

        login_button.connect_clicked(callback);
    }
}
