// Source https://github.com/gtk-rs/glib/blob/master/src/clone.rs
// To be removed once there's a new webkit2gtk-rs release
use std::rc::{self, Rc};
use std::sync::{self, Arc};

/// Trait for generalizing downgrading a strong reference to a weak reference.
pub trait Downgrade
where
    Self: Sized,
{
    /// Weak reference type.
    type Weak;

    /// Downgrade to a weak reference.
    fn downgrade(&self) -> Self::Weak;
}

/// Trait for generalizing upgrading a weak reference to a strong reference.
pub trait Upgrade
where
    Self: Sized,
{
    /// Strong reference type.
    type Strong;

    /// Try upgrading a weak reference to a strong reference.
    fn upgrade(&self) -> Option<Self::Strong>;
}

impl<T: Downgrade + glib::ObjectType> Upgrade for glib::WeakRef<T> {
    type Strong = T;

    fn upgrade(&self) -> Option<Self::Strong> {
        self.upgrade()
    }
}

impl<T: Downgrade> Downgrade for &T {
    type Weak = T::Weak;

    fn downgrade(&self) -> Self::Weak {
        T::downgrade(*self)
    }
}

impl<T> Downgrade for Arc<T> {
    type Weak = sync::Weak<T>;

    fn downgrade(&self) -> Self::Weak {
        Arc::downgrade(self)
    }
}

impl<T> Upgrade for sync::Weak<T> {
    type Strong = Arc<T>;

    fn upgrade(&self) -> Option<Self::Strong> {
        self.upgrade()
    }
}

impl<T> Downgrade for Rc<T> {
    type Weak = rc::Weak<T>;

    fn downgrade(&self) -> Self::Weak {
        Rc::downgrade(self)
    }
}

impl<T> Upgrade for rc::Weak<T> {
    type Strong = Rc<T>;

    fn upgrade(&self) -> Option<Self::Strong> {
        self.upgrade()
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! to_type_before {
    (_) => ();
    ($($variable:ident).+ $(as $rename:ident)?) => (
        // In case we have:
        // clone!(v => move || {});
        compile_error!("You need to specify if this is a weak or a strong clone.");
    );
    (@strong $variable:ident) => (
        let $variable = $variable.clone();
    );
    (@weak $variable:ident) => (
        let $variable = $crate::clone::Downgrade::downgrade(&$variable);
    );
    (@strong $($variable:ident).+ as $rename:ident) => (
        let $rename = $($variable).+.clone();
    );
    (@weak $($variable:ident).+ as $rename:ident) => (
        let $rename = $crate::clone::Downgrade::downgrade(&$($variable).+);
    );
    (@ $keyword:ident $($variable:ident).+ $(as $rename:ident)?) => (
        // In case we have:
        // clone!(@yolo v => move || {});
        compile_error!("Unknown keyword, only `weak` and `strong` are allowed");
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! to_type_after {
    (@default-panic, @weak $variable:ident) => {
        let $variable = match $crate::clone::Upgrade::upgrade(&$variable) {
            Some(val) => val,
            None => panic!("failed to upgrade {}", stringify!($variable)),
        };
    };
    (as $rename:ident @default-panic, @weak $($variable:ident).+) => {
        let $rename = match $crate::clone::Upgrade::upgrade(&$rename) {
            Some(val) => val,
            None => panic!("failed to upgrade {}", stringify!($rename)),
        };
    };
    ($(as $rename:ident)? @default-panic, @strong $($variable:ident).+) => {};
    (@weak $variable:ident , $return_value:expr) => {
        let $variable = match $crate::clone::Upgrade::upgrade(&$variable) {
            Some(val) => val,
            None => return ($return_value)(),
        };
    };
    (as $rename:ident @weak $($variable:ident).+ , $return_value:expr) => {
        let $rename = match $crate::clone::Upgrade::upgrade(&$rename) {
            Some(val) => val,
            None => return ($return_value)(),
        };
    };
    ($(as $rename:ident)? @strong $($variable:ident).+ , $return_value:expr) => {};
    ($(as $rename:ident)? @ $keyword:ident $($variable:ident).+, $return_value:expr) => {};
}

#[doc(hidden)]
#[macro_export]
macro_rules! to_return_value {
    () => {
        ()
    };
    ($value:expr) => {
        $value
    };
}

#[macro_export]
macro_rules! clone {
    ( => $($_:tt)*) => (
        // In case we have:
        // clone!( => move || {});
        compile_error!("If you have nothing to clone, no need to use this macro!");
    );
    ($(move)? || $($_:tt)*) => (
        // In case we have:
        // clone!(|| {});
        compile_error!("If you have nothing to clone, no need to use this macro!");
    );
    ($(move)? | $($arg:tt $(: $typ:ty)?),* | $($_:tt)*) => (
        // In case we have:
        // clone!(|a, b| {});
        compile_error!("If you have nothing to clone, no need to use this macro!")
    );
    ($($(@ $strength:ident)? self),+ => $($_:tt)* ) => (
        compile_error!("Can't use `self` as variable name. Try storing it in a temporary variable or rename it using `as`.");
    );
    ($($(@ $strength:ident)? $up:ident.$($variables:ident).+),+ => $($_:tt)* ) => (
        compile_error!("Field accesses are not allowed as is, you must rename it!");
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-panic, move || $body:block ) => (
        {
            $( $crate::to_type_before!($(@ $strength)? $($variables).+ $(as $rename)?); )*
            move || {
                $( $crate::to_type_after!($(as $rename)? @default-panic, $(@ $strength)? $($variables).+);)*
                $body
            }
        }
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-panic, move || $body:expr ) => (
        clone!($($(@ $strength)? $($variables).+ $(as $rename)?),* => @default-panic, move || { $body })
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => $(@default-return $return_value:expr,)? move || $body:block ) => (
        {
            $( $crate::to_type_before!($(@ $strength)? $($variables).+ $(as $rename)?); )*
            move || {
                let _return_value = || $crate::to_return_value!($($return_value)?);
                $( $crate::to_type_after!($(as $rename)? $(@ $strength)? $($variables).+, _return_value );)*
                $body
            }
        }
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => $(@default-return $return_value:expr,)? move || $body:expr ) => (
        clone!($($(@ $strength)? $($variables).+ $(as $rename)?),* => $(@default-return $return_value,)? move || { $body })
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-panic, move | $($arg:tt $(: $typ:ty)?),* | $body:block ) => (
        {
            $( $crate::to_type_before!($(@ $strength)? $($variables).+ $(as $rename)?); )*
            move |$($arg $(: $typ)?),*| {
                $( $crate::to_type_after!($(as $rename)? @default-panic, $(@ $strength)? $($variables).+);)*
                $body
            }
        }
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-panic, move | $($arg:tt $(: $typ:ty)?),* | $body:expr ) => (
        clone!($($(@ $strength)? $($variables).+ $(as $rename)?),* => @default-panic, move |$($arg $(: $typ)?),*| { $body })
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => $(@default-return $return_value:expr,)? move | $($arg:tt $(: $typ:ty)?),* | $body:block ) => (
        {
            $( $crate::to_type_before!($(@ $strength)? $($variables).+ $(as $rename)?); )*
            move | $($arg $(: $typ)?),* | {
                let _return_value = || $crate::to_return_value!($($return_value)?);
                $( $crate::to_type_after!($(as $rename)? $(@ $strength)? $($variables).+, _return_value);)*
                $body
            }
        }
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => $(@default-return $return_value:expr,)? move | $($arg:tt $(: $typ:ty)?),* | $body:expr ) => (
        clone!($($(@ $strength)? $($variables).+ $(as $rename)?),+ => $(@default-return $return_value,)? move |$($arg $(: $typ)?),*| { $body })
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-return $return_value:expr, || $body:block ) => (
        // In case we have:
        // clone!(@weak foo => @default-return false, || {});
        compile_error!("Closure needs to be \"moved\" so please add `move` before closure");
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-return $return_value:expr, | $($arg:tt $(: $typ:ty)?),* | $body:block ) => (
        // In case we have:
        // clone!(@weak foo => @default-return false, |bla| {});
        compile_error!("Closure needs to be \"moved\" so please add `move` before closure");
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => default-return $($x:tt)+ ) => (
        // In case we have:
        // clone!(@weak foo => default-return false, move || {});
        compile_error!("Missing `@` before `default-return`");
    );
    ($($(@ $strength:ident)? $($variables:ident).+ $(as $rename:ident)?),+ => @default-return $($x:tt)+ ) => (
        // In case we have:
        // clone!(@weak foo => @default-return false move || {});
        compile_error!("Missing comma after `@default-return`'s value");
    );
    ($($(@ $strength:ident)? $variables:expr),+ => $($_:tt)* ) => (
        compile_error!("Variables need to be valid identifiers, e.g. field accesses are not allowed as is, you must rename it!");
    );
}
