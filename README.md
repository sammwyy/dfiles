# File Explorer

> A calm, native file explorer for Linux.

**File Explorer** (`dfiles`) is designed to feel like part of your desktop, not
like a branded destination. It follows the system theme, uses the folders and
mounts already configured on your machine, and stays out of the way while you
navigate, open, save, and organize files.

[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)

## Highlights

- **At home on your desktop** — respects the system appearance, including live
  theme updates where a CreamUI-compatible theme service is available.
- **Straightforward navigation** — browse your home directory, locations from
  `user-dirs.dirs`, mounted drives, local paths, and `file://` URIs.
- **Built for everyday work** — tabs, address-bar navigation, search, pinned
  folders, recently opened files, and storage information in one quiet window.
- **Uses your defaults** — opens files through `xdg-open`, so the application
  you chose remains the application that opens the file.
- **Desktop integration** — includes a `.desktop` entry and an optional XDG
  Desktop Portal FileChooser backend for open, save, and folder-selection
  dialogs.
- **No account, sync service, or vendor lock-in** — your files and preferences
  remain on your system.

## Requirements

File Explorer currently targets Linux desktops. To build it, install the stable
Rust toolchain, a C compiler, `pkg-config`, and the native development
libraries used by the desktop backend.

For example:

```bash
# Arch Linux
sudo pacman -S --needed base-devel rust pkgconf dbus wayland libxkbcommon

# Debian / Ubuntu
sudo apt install build-essential cargo pkg-config libdbus-1-dev \
  libwayland-dev libxkbcommon-dev

# Fedora
sudo dnf install gcc-c++ cargo pkgconf-pkg-config dbus-devel \
  wayland-devel libxkbcommon-devel
```

## Install from source

Clone the repository and build an optimized binary:

```bash
git clone https://github.com/sammwyy/dfiles.git
cd dfiles
cargo build --release --locked
```

Run it directly, optionally passing a folder or local file URI:

```bash
./target/release/dfiles
./target/release/dfiles ~/Documents
./target/release/dfiles file:///home/me/Downloads
```

For a system-wide installation under `/usr/local`, install the binary and
desktop entry. The portal service files below are optional and only needed if
you want File Explorer to provide file chooser dialogs to other applications.

```bash
sudo install -Dm755 target/release/dfiles /usr/local/bin/dfiles
sudo install -Dm644 data/dfiles.desktop \
  /usr/local/share/applications/dfiles.desktop

sudo install -d /usr/local/lib/systemd/user /usr/local/share/dbus-1/services
sed 's#/usr/bin/dfiles#/usr/local/bin/dfiles#g' \
  data/dfiles-portal.service \
  | sudo tee /usr/local/lib/systemd/user/dfiles-portal.service >/dev/null
sed 's#/usr/bin/dfiles#/usr/local/bin/dfiles#g' \
  data/org.freedesktop.impl.portal.desktop.dfiles.service \
  | sudo tee /usr/local/share/dbus-1/services/org.freedesktop.impl.portal.desktop.dfiles.service >/dev/null
sudo install -Dm644 data/dfiles.portal \
  /usr/local/share/xdg-desktop-portal/portals/dfiles.portal
```

Log out and back in (or restart `xdg-desktop-portal`) after installing the
portal files so the desktop can discover the backend.

File Explorer is a portal backend, not a replacement for
`xdg-desktop-portal`. Your desktop or distribution's portal policy chooses
whether to use it; keep other installed backends in that policy as fallbacks.

## Usage

```text
dfiles [FOLDER | file://URI]
dfiles --settings
dfiles --portal
```

| Command | What it does |
| --- | --- |
| `dfiles` | Opens File Explorer at Home. |
| `dfiles /path/to/folder` | Opens a particular folder. |
| `dfiles file:///path/to/folder` | Opens a local `file://` URI. |
| `dfiles --settings` | Opens File Explorer settings. `--setings` is accepted as a compatibility alias. |
| `dfiles --portal` | Starts the XDG Desktop Portal FileChooser backend. |
| `dfiles --help` | Shows the available command-line options. |

Settings are stored in `$XDG_CONFIG_HOME/dfiles/config.toml` (normally
`~/.config/dfiles/config.toml`).

The `explorer.tab_position` preference accepts `"top"` (the default) or
`"bottom"`; `explorer.show_statusbar` controls the item-count/status area.
Both are also available from `dfiles --settings` and apply to newly opened
windows.

## Development

```bash
cargo test --locked
cargo run -- /path/to/a/folder
```

CreamUI is pinned to a public commit in `Cargo.toml`, so a normal clone builds
without a sibling checkout. Update that revision deliberately when adopting a
new CreamUI version, then refresh `Cargo.lock`.

## Releases

Pushing any tag (for example, `v0.1.0`) creates a GitHub Actions artifact named
`dfiles-v0.1.0-x86_64-unknown-linux-gnu`. It contains the release binary,
compressed with UPX. The workflow builds from `Cargo.lock` and verifies the
test suite before publishing the artifact.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
