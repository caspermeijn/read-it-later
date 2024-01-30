# Profiling

## Install dependencies

```bash
flatpak install org.gnome.Sdk.Debug
cargo install samply
```

### Build and Record

Build the application by choosing `Flatpak: Build & Run` from the command palette in VSCode.

Then profile it: open `Flatpak: Open a build terminal` from the command palette in VSCode and run:

```bash
samply record /app/bin/read-it-later
```
