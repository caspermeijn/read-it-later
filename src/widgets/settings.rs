use crate::models::ClientManager;
use crate::settings::{Key, SettingsManager};
use async_std::sync::{Arc, Mutex};
use gtk::prelude::*;
use gtk_macros::{get_widget, send, spawn};
use log::error;
use std::rc::Rc;

struct ClientInfo {
    pub username: String,
    pub email: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

enum SettingsAction {
    ClientInfoLoaded(ClientInfo),
}

pub struct SettingsWidget {
    pub widget: libhandy::PreferencesWindow,
    builder: gtk::Builder,
}

impl SettingsWidget {
    pub fn new(client: Arc<Mutex<ClientManager>>) -> Rc<Self> {
        let builder = gtk::Builder::from_resource("/com/belmoussaoui/ReadItLater/settings.ui");
        get_widget!(builder, libhandy::PreferencesWindow, settings_window);

        let window = Rc::new(Self {
            builder,
            widget: settings_window,
        });

        window.init(window.clone(), client);
        window.setup_signals();
        window
    }

    fn init(&self, settings: Rc<Self>, client: Arc<Mutex<ClientManager>>) {
        let (sender, receiver) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        receiver.attach(None, move |action| settings.do_action(action));

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
        get_widget!(self.builder, gtk::Label, username_label);
        get_widget!(self.builder, gtk::Label, email_label);
        get_widget!(self.builder, gtk::Label, created_at_label);
        get_widget!(self.builder, gtk::Label, updated_at_label);

        match action {
            SettingsAction::ClientInfoLoaded(client_info) => {
                username_label.set_text(&client_info.username);
                email_label.set_text(&client_info.email);
                if let Some(created_at) = client_info.created_at {
                    created_at_label.set_text(&created_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
                if let Some(updated_at) = client_info.updated_at {
                    updated_at_label.set_text(&updated_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
            }
        }

        glib::Continue(true)
    }
    fn setup_signals(&self) {
        get_widget!(self.builder, gtk::Switch, dark_mode_button);
        SettingsManager::bind_property(Key::DarkMode, &dark_mode_button, "active");
    }
}
