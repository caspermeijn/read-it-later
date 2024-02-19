// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2021 Alistair Francis <alistair@alistair23.me>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use adw::subclass::prelude::*;
use async_std::sync::{Arc, Mutex};
use gtk::glib;

use crate::models::ClientManager;

mod imp {
    use glib::subclass::InitializingObject;

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

    impl ObjectImpl for SettingsWidget {
        fn dispose(&self) {
            self.dispose_template();
        }
    }

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
        let window = glib::Object::new::<Self>();
        window.init(client);
        window
    }

    fn init(&self, client: Arc<Mutex<ClientManager>>) {
        let (sender, receiver) = async_std::channel::unbounded();
        let ctx = glib::MainContext::default();

        ctx.spawn_local(glib::clone!(@strong self as window =>  async move {
            while let Ok(action) = receiver.recv().await {
                window.do_action(action);
            }
        }));

        ctx.spawn_local(async move {
            let client = client.lock().await;
            if let Ok(user) = client.fetch_user().await {
                sender
                    .send(SettingsAction::ClientInfoLoaded(ClientInfo {
                        username: user.username.clone(),
                        email: user.email.clone(),
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    }))
                    .await
                    .unwrap();
            }
        });
    }

    fn do_action(&self, action: SettingsAction) -> glib::ControlFlow {
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

        glib::ControlFlow::Continue
    }
}
