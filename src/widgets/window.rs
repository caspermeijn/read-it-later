use gtk::prelude::*;
use libhandy::prelude::*;

use crate::config::{APP_ID, PROFILE};
use crate::window_state;

pub struct Window {
    pub widget: gtk::ApplicationWindow,
    builder: gtk::Builder,
}

impl Window {
    pub fn new() -> Self {
        let settings = gio::Settings::new(APP_ID);
        let builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/window.ui");
        let window_widget: gtk::ApplicationWindow = builder.get_object("window").unwrap();

        if PROFILE == "Devel" {
            window_widget.get_style_context().add_class("devel");
        }

        let window = Window {
            widget: window_widget,
            builder,
        };

        window.init(settings);
        window
    }

    pub fn init(&self, settings: gio::Settings) {
        // setup app menu
        let menu_builder = gtk::Builder::new_from_resource("/com/belmoussaoui/ReadItLater/menu.ui");
        let popover_menu: gtk::PopoverMenu = menu_builder.get_object("popover_menu").unwrap();
        let appmenu_btn: gtk::MenuButton = self.builder.get_object("appmenu_button").unwrap();
        appmenu_btn.set_popover(Some(&popover_menu));
        // load latest window state
        window_state::load(&self.widget, &settings);

        // save window state on delete event
        self.widget.connect_delete_event(move |window, _| {
            window_state::save(&window, &settings);
            Inhibit(false)
        });

        let squeezer: libhandy::Squeezer = self.builder.get_object("squeezer").unwrap();
        let switcher_bar: libhandy::ViewSwitcherBar = self.builder.get_object("switcher_bar").unwrap();

        let title_wide_switcher: libhandy::ViewSwitcher = self.builder.get_object("title_wide_switcher").unwrap();
        let title_narrow_switcher: libhandy::ViewSwitcher = self.builder.get_object("title_narrow_switcher").unwrap();
        let title_label: gtk::Label = self.builder.get_object("title_label").unwrap();

        self.widget.connect_size_allocate(move |_, allocation| {
            squeezer.set_child_enabled(&title_wide_switcher, allocation.width > 600);
            squeezer.set_child_enabled(&title_label, allocation.width <= 450);
            squeezer.set_child_enabled(&title_narrow_switcher, allocation.width > 450);
            switcher_bar.set_reveal(allocation.width <= 450);
        });
    }
}
