use failure::Error;
use gio::prelude::*;
// Stolen from Shortwave
macro_rules! get_widget {
    ($builder:expr, $wtype:ty, $name:ident) => {
        let $name: $wtype = $builder
            .get_object(stringify!($name))
            .expect(&format!("Could not find widget \"{}\"", stringify!($name)));
    };
}

macro_rules! spawn {
    ($future:expr) => {
        let ctx = glib::MainContext::default();
        ctx.spawn($future);
    };
}

macro_rules! send {
    ($sender:expr, $action:expr) => {
        if let Err(err) = $sender.send($action) {
            error!("Failed to send \"{}\" action due to {}", stringify!($action), err);
        }
    };
}

macro_rules! action {
    ($actions_group:expr, $name:expr, $callback:expr) => {
        let simple_action = gio::SimpleAction::new($name, None);
        simple_action.connect_activate($callback);
        $actions_group.add_action(&simple_action);
    };
}

macro_rules! stateful_action {
    ($actions_group:expr, $name:expr, $value:expr, $callback:expr) => {
        let simple_action = gio::SimpleAction::new_stateful($name, None, &$value.to_variant());
        simple_action.connect_activate($callback);
        $actions_group.add_action(&simple_action);
    };
}

pub fn load_resource(file: &str) -> Result<String, Error> {
    let file = gio::File::new_for_uri(&format!("resource:///com/belmoussaoui/ReadItLater/{}", file));
    let (bytes, _) = file.load_bytes(gio::NONE_CANCELLABLE)?;
    String::from_utf8(bytes.to_vec()).map_err(From::from)
}
