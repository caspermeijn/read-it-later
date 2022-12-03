use gtk::{glib, prelude::*, subclass::prelude::*};
use log::error;

use crate::models::Account;

mod imp {
    use glib::subclass::InitializingObject;

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/login.ui")]
    pub struct Login {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[template_child]
        pub login_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub instance_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub client_id_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub client_secret_entry: TemplateChild<adw::PasswordEntryRow>,

        #[template_child]
        pub username_entry: TemplateChild<adw::EntryRow>,

        #[template_child]
        pub password_entry: TemplateChild<adw::PasswordEntryRow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn constructed(&self) {
            self.parent_constructed();

            self.icon
                .set_icon_name(Some(&format!("{}-symbolic", crate::config::APP_ID)));
        }

        fn dispose(&self) {
            self.obj().dispose_template(Self::Type::static_type());
        }
    }

    impl WidgetImpl for Login {}
}

glib::wrapper! {
    pub struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget;
}

#[gtk::template_callbacks]
impl Login {
    #[template_callback]
    fn login_button_clicked(&self, button: &gtk::Button) {
        let account = self.get_account();

        if let Some(account) = account {
            button
                .activate_action("app.login", Some(&account.to_variant()))
                .expect("The action does not exist.");
        }
    }

    pub fn get_account(&self) -> Option<Account> {
        let instance = self.imp().instance_entry.text();
        let instance = instance.trim_end_matches('/').to_string();
        if let Err(err) = url::Url::parse(&instance) {
            error!("The instance url is invalid {}", err);
            self.imp().instance_entry.add_css_class("error");
            return None;
        }
        self.imp().instance_entry.remove_css_class("error");

        Some(Account {
            instance_url: instance,
            client_id: self.imp().client_id_entry.text().to_string(),
            client_secret: self.imp().client_secret_entry.text().to_string(),
            username: self.imp().username_entry.text().to_string(),
            password: self.imp().password_entry.text().to_string(),
        })
    }

    pub fn get_login_button(&self) -> &gtk::Button {
        &self.imp().login_button
    }
}

impl Default for Login {
    fn default() -> Self {
        glib::Object::new(&[])
    }
}
