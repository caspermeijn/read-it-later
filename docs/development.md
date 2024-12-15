<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright 2024 Casper Meijn <casper@meijn.net> -->

# Development setup

## Ubuntu 24.04

- Install VSCode
- Install recommended VSCode extensions
- Install flatpak: `sudo apt install flatpak`
- Add flathub repository: `flatpak remote-add --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`
- Install required flatpak SDK: `flatpak install org.gnome.Sdk//47 org.freedesktop.Sdk.Extension.rust-stable//24.08 org.freedesktop.Sdk.Extension.llvm18//24.08`
- In VSCode: press F1 and select `Flatpak: Build and Run`
