use glib::subclass::InitializingObject;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use log::error;
use wallabag_api::types::Config;

mod imp {
    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/login.ui")]
    pub struct Login {
        #[template_child]
        pub icon: TemplateChild<gtk::Image>,

        #[template_child]
        pub login_button: TemplateChild<gtk::Button>,

        #[template_child]
        pub instance_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub client_id_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub client_secret_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub username_entry: TemplateChild<gtk::Entry>,

        #[template_child]
        pub password_entry: TemplateChild<gtk::Entry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Login {
        const NAME: &'static str = "Login";
        type Type = super::Login;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for Login {
        fn constructed(&self, obj: &Self::Type) {
            // Call "constructed" on parent
            self.parent_constructed(obj);

            self.icon.set_icon_name(Some(&format!("{}-symbolic", crate::config::APP_ID)));
        }
    }

    impl WidgetImpl for Login {}
}

glib::wrapper! {
    pub struct Login(ObjectSubclass<imp::Login>)
        @extends gtk::Widget;
}

impl Login {
    pub fn new() -> Self {
        Object::new(&[]).expect("Failed to create Window")
    }

    pub fn on_login_clicked<F>(&self, callback: F)
    where
        for<'r> F: std::ops::Fn(&'r gtk::Button) + 'static,
    {
        self.imp().login_button.connect_clicked(callback);
    }

    pub fn get_wallabag_client_config(&self) -> Option<Config> {
        let instance = self.imp().instance_entry.text();
        let instance = instance.trim_end_matches('/').to_string();
        if let Err(err) = url::Url::parse(&instance) {
            error!("The instance url is invalid {}", err);
            self.imp().instance_entry.add_css_class("error");
            return None;
        }
        self.imp().instance_entry.remove_css_class("error");

        Some(Config {
            client_id: self.imp().client_id_entry.text().to_string(),
            client_secret: self.imp().client_secret_entry.text().to_string(),
            username: self.imp().username_entry.text().to_string(),
            password: self.imp().password_entry.text().to_string(),
            base_url: instance,
        })
    }

    pub fn get_login_button(&self) -> &gtk::Button {
        &self.imp().login_button
    }
}
