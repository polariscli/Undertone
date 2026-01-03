# Undertone

**Linux-native audio mixer for Elgato Wave:3** - providing Wave Link-style functionality using PipeWire.

Undertone gives you independent control over multiple audio channels with separate stream and monitor mixes, perfect for streamers and content creators on Linux.

## Features

- **5 Audio Channels** - System, Voice, Music, Browser, Game
- **Dual Mix Architecture** - Separate Stream and Monitor mixes with independent volume/mute per channel
- **Automatic App Routing** - Apps route to channels based on configurable rules (Discord → Voice, Spotify → Music, etc.)
- **Master Volume Control** - Per-mix master volume and mute
- **Output Device Selection** - Route monitor mix to any audio output (headphones, speakers, HDMI)
- **Profiles** - Save and load mixer configurations
- **Mic Control** - Gain and mute control for Wave:3 microphone
- **Native UI** - Qt6/QML with KDE Kirigami theming

## Screenshots

*Coming soon*

## Requirements

- Linux with PipeWire (Fedora 43+, Ubuntu 24.04+, Arch, etc.)
- Elgato Wave:3 microphone (optional - works as general audio mixer too)
- Rust 1.85+ (Edition 2024)
- Qt6 with Kirigami

## Installation

### Dependencies

```bash
# Fedora
sudo dnf install pipewire-devel qt6-qtbase-devel qt6-qtdeclarative-devel \
    clang kf6-kirigami-devel kf6-qqc2-desktop-style

# Arch Linux
sudo pacman -S pipewire qt6-base qt6-declarative clang kirigami

# Ubuntu/Debian
sudo apt install libpipewire-0.3-dev qt6-base-dev qt6-declarative-dev \
    clang libkf6kirigami-dev
```

### Build

```bash
git clone https://github.com/afterlike/undertone.git
cd undertone
cargo build --release
```

### Run

```bash
# Start the daemon (required)
cargo run -p undertone-daemon --release

# In another terminal, start the UI
cargo run -p undertone-ui --release
```

### Install (optional)

```bash
# Install binaries, systemd service, and udev rules
./scripts/install.sh
```

## How It Works

Undertone creates virtual audio sinks in PipeWire that applications connect to. Each channel feeds into volume filter nodes that control the audio level independently for Stream and Monitor mixes.

```
App (Spotify)
  -> ut-ch-music (channel sink)
       -> ut-ch-music-stream-vol -> ut-stream-mix -> OBS
       -> ut-ch-music-monitor-vol -> ut-monitor-mix -> Headphones
```

### Default App Routing

| Pattern    | Channel  |
| ---------- | -------- |
| discord    | Voice    |
| zoom       | Voice    |
| teams      | Voice    |
| spotify    | Music    |
| rhythmbox  | Music    |
| firefox    | Browser  |
| chrome     | Browser  |
| steam      | Game     |
| *default*  | System   |

## Usage

### Mixer Tab
- Adjust volume sliders to control audio levels
- Click mute button to silence a channel
- Toggle between Stream and Monitor mix views
- Use master volume for overall mix control

### Apps Tab
- View currently playing audio applications
- Click channel dropdown to reassign apps
- Routes are automatically saved

### Device Tab
- View Wave:3 connection status
- Adjust microphone gain
- Toggle mic mute

### Profiles
- Click profile name in header to switch profiles
- Use menu to save current settings or delete profiles

## Configuration

Data is stored in `~/.local/share/undertone/`:
- `undertone.db` - SQLite database with channels, routes, profiles
- Logs via systemd journal when running as service

WirePlumber configuration for Wave:3 naming:
- `~/.config/wireplumber/wireplumber.conf.d/51-elgato.conf`

## Troubleshooting

### No audio from channels
```bash
# Check if daemon is running
pgrep undertone-daemon

# Verify PipeWire nodes exist
pw-cli list-objects Node | grep ut-

# Check audio links
pw-link -l | grep ut-
```

### App routing to wrong channel
```bash
# Check database routes
sqlite3 ~/.local/share/undertone/undertone.db "SELECT * FROM app_routes;"

# Restart daemon to re-apply routes
pkill undertone-daemon && cargo run -p undertone-daemon
```

### UI not connecting
```bash
# Check socket exists
ls -la $XDG_RUNTIME_DIR/undertone/daemon.sock

# Test IPC
echo '{"id":1,"method":{"type":"GetState"}}' | \
    socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/undertone/daemon.sock
```

## Architecture

**undertone-daemon** (background service)
- IPC Server (Unix socket) | Signal Handler | Event Loop (Tokio)
- **undertone-core**: Channels, Mixer, App Routing, Profiles, State
- **undertone-pipewire**: PipeWire graph management
- **undertone-db**: SQLite persistence
- **undertone-hid**: Wave:3 hardware (ALSA fallback)

*Unix Socket IPC*

**undertone-ui** (Qt6/QML + Kirigami + cxx-qt)

## Contributing

Contributions welcome! Please see [PROGRESS.md](PROGRESS.md) for current status and planned features.

## License

MIT OR Apache-2.0

## Acknowledgments

- [pipewire-rs](https://gitlab.freedesktop.org/pipewire/pipewire-rs) - Rust bindings for PipeWire
- [cxx-qt](https://github.com/KDAB/cxx-qt) - Safe Rust/Qt interop
- [KDE Kirigami](https://develop.kde.org/frameworks/kirigami/) - UI framework
- Elgato for the excellent Wave:3 hardware
