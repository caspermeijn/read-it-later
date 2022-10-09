pub struct LoginView {
    pub widget: crate::widgets::Login,
    pub name: String,
}

impl LoginView {
    pub fn new() -> Self {
        let widget = crate::widgets::Login::new();

        Self {
            widget,
            name: "login".to_string(),
        }
    }
}
