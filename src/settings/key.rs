#[derive(Display, Debug, Clone, EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum Key {
    /* User Interface */
    DarkMode,
    WindowWidth,
    WindowHeight,
    ViewSorting,
    ViewOrder,
}
