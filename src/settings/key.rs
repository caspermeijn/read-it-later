// Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
// Copyright 2022 Casper Meijn <casper@meijn.net>
//
// SPDX-License-Identifier: GPL-3.0-or-later

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
