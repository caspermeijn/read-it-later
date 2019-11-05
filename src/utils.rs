use failure::Error;

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
        match $sender.send($action) {
            Err(err) => error!("Failed to send \"{}\" action due to {}", stringify!($action), err),
            _ => (),
        };
    };
}

// Source: https://github.com/gtk-rs/examples/
// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

pub fn load_resource(file: &str) -> Result<String, Error> {
    use gio::FileExt;
    let file = gio::File::new_for_uri(&format!("resource:///com/belmoussaoui/ReadItLater/{}", file));
    let (bytes, _) = file.load_bytes(gio::NONE_CANCELLABLE)?;
    String::from_utf8(bytes.to_vec()).map_err(From::from)
}
