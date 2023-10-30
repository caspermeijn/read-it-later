#!/bin/bash

# Copyright 2019 Bilal Elmoussaoui <bil.elmoussaoui@gmail.com>
#
# SPDX-License-Identifier: GPL-3.0-or-later

export DIST="$1"
export SOURCE_ROOT="$2"

cd "$SOURCE_ROOT"
mkdir "$DIST"/.cargo
cargo vendor | sed 's/^directory = ".*"/directory = "vendor"/g' > $DIST/.cargo/config
# Move vendor into dist tarball directory
mv vendor "$DIST"

