<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright 2024 Casper Meijn <casper@meijn.net> -->

# Creating a release
- Add a tag to the master branch
- Checkout the tag in your local repo
- Select `Flatpak: Open a Build Terminal` from the Command Pallette in VSCode
- Run `ninja -C _build dist` to create a source tarball
- Create a Gitlab release
- Upload the source tarball to the release
