// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod article;
pub mod articles;
mod new_article;
mod settings;
mod window;

pub use article::ArticleWidget;
pub use settings::SettingsWidget;
pub use window::{View, Window};
