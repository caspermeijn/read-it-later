use adw::subclass::prelude::*;
use async_std::sync::{Arc, Mutex};
use gtk::glib;
use gtk_macros::{send, spawn};
use log::error;

use crate::models::ClientManager;

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::prelude::*;

    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/com/belmoussaoui/ReadItLater/settings.ui")]
    pub struct SettingsWidget {
        #[template_child]
        pub username_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub email_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub created_at_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub updated_at_label: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWidget {
        const NAME: &'static str = "SettingsWidget";
        type Type = super::SettingsWidget;
        type ParentType = adw::PreferencesWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SettingsWidget {}

    impl WidgetImpl for SettingsWidget {}

    impl WindowImpl for SettingsWidget {}

    impl AdwWindowImpl for SettingsWidget {}

    impl PreferencesWindowImpl for SettingsWidget {}
}

glib::wrapper! {
    pub struct SettingsWidget(ObjectSubclass<imp::SettingsWidget>)
        @extends gtk::Widget, gtk::Window, adw::Window, adw::PreferencesWindow;
}

struct ClientInfo {
    pub username: String,
    pub email: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

enum SettingsAction {
    ClientInfoLoaded(ClientInfo),
}

impl SettingsWidget {
    pub fn new(client: Arc<Mutex<ClientManager>>) -> Self {
        let window = glib::Object::new::<Self>(&[]);
        window.init(client);
        window
    }

    fn init(&self, client: Arc<Mutex<ClientManager>>) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        receiver.attach(
            None,
            glib::clone!(@strong self as window =>  move |action| {
                    window.do_action(action)
            }),
        );

        spawn!(async move {
            let client = client.lock().await;
            if let Ok(user) = client.fetch_user().await {
                send!(
                    sender,
                    SettingsAction::ClientInfoLoaded(ClientInfo {
                        username: user.username.clone(),
                        email: user.email.clone(),
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    })
                );
            }
        });
    }

    fn do_action(&self, action: SettingsAction) -> glib::Continue {
        match action {
            SettingsAction::ClientInfoLoaded(client_info) => {
                let imp = self.imp();
                imp.username_label.set_text(&client_info.username);
                imp.email_label.set_text(&client_info.email);
                if let Some(created_at) = client_info.created_at {
                    imp.created_at_label
                        .set_text(&created_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
                if let Some(updated_at) = client_info.updated_at {
                    imp.updated_at_label
                        .set_text(&updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
            }
        }

        glib::Continue(true)
    }
}
