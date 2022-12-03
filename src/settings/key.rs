use strum::{Display, EnumString};

#[derive(Display, Debug, Clone, EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum Key {
    // User Interface
    Username,   // username
    LatestSync, // latest-sync

    WindowWidth,
    WindowHeight,
    WindowX,
    WindowY,
    IsMaximized,
}
