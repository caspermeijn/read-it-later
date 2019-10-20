// Stolen from Shortwave
macro_rules! get_widget {
    ($builder:expr, $wtype:ty, $name:ident) => {
        let $name: $wtype = $builder
            .get_object(stringify!($name))
            .expect(&format!("Could not find widget \"{}\"", stringify!($name)));
    };
}
