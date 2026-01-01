# Undertone Development Progress

**Project**: Linux-native Elgato Wave:3 audio control application
**License**: MIT / Apache-2.0 (Open Source)
**Started**: January 2026

---

## Current Status: Milestone 4 Complete

The IPC protocol is now fully implemented with request/response handling and event broadcasting. The daemon handles volume/mute changes, app routing, and emits events for state changes. State mutations are tracked in memory with database persistence for routing rules.

### Verified Working Features

```bash
# Virtual nodes created
$ pw-cli list-objects Node | grep -E "ut-|wave3"
node.name = "wave3-source"
node.name = "wave3-null-sink"
node.name = "wave3-sink"
node.name = "ut-ch-system"
node.name = "ut-ch-voice"
node.name = "ut-ch-music"
node.name = "ut-ch-browser"
node.name = "ut-ch-game"
node.name = "ut-stream-mix"
node.name = "ut-monitor-mix"

# Links connecting channels to mixes (20 links total)
# Each channel has 4 links: 2 to stream-mix, 2 to monitor-mix (stereo)
$ pw-link -l | grep "ut-"

# IPC socket created and responding
$ ls -la $XDG_RUNTIME_DIR/undertone/daemon.sock
srw-rw-rw- 1 user user 0 Jan  1 12:00 /run/user/1000/undertone/daemon.sock

# Example IPC request/response
$ echo '{"id":1,"method":{"type":"GetState"}}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/undertone/daemon.sock
{"id":1,"result":{"Ok":{"state":"running","device_connected":true,...}}}
```

---

## System Context

| Component    | Version                                                   |
| ------------ | --------------------------------------------------------- |
| OS           | Fedora 43, Linux 6.17.12                                  |
| PipeWire     | 1.4.9 (socket-activated)                                  |
| WirePlumber  | 0.5.12                                                    |
| Rust         | 1.92.0                                                    |
| Wave:3       | VID 0x0fd9, PID 0x0070                                    |
| Existing Fix | `~/.config/wireplumber/wireplumber.conf.d/51-elgato.conf` |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        undertone-daemon                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Unix Socket  │  │   Signal     │  │    Config Watcher    │   │
│  │   Server     │  │   Handler    │  │      (notify)        │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│                           │                                      │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                    undertone-core                           │ │
│  │  Channels │ Mixer (Stream/Monitor) │ App Routing │ State   │ │
│  └─────────────────────────────────────────────────────────────┘ │
│         │                 │                 │                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐   │
│  │ undertone-  │  │ undertone-  │  │     undertone-hid       │   │
│  │  pipewire   │  │     db      │  │       (plugin)          │   │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Project Structure

```
Undertone/
├── Cargo.toml                    # Workspace root
├── PROGRESS.md                   # This file
├── crates/
│   ├── undertone-daemon/         # Main daemon binary ✅
│   ├── undertone-core/           # Business logic ✅
│   ├── undertone-pipewire/       # PipeWire graph management ✅
│   ├── undertone-db/             # SQLite persistence ✅
│   ├── undertone-ipc/            # IPC protocol definitions ✅
│   ├── undertone-hid/            # Wave:3 HID integration (stub)
│   └── undertone-ui/             # UI application (stub)
├── ui/                           # QML UI definitions (not yet created)
├── config/                       # Config templates (not yet created)
└── scripts/                      # Install scripts (not yet created)
```

---

## Milestone Progress

### ✅ Milestone 1: Foundation (COMPLETE)

**Deliverables:**

- [x] Rust workspace with all crate scaffolding
- [x] Daemon connects to PipeWire, logs events
- [x] SQLite schema and migrations
- [x] Basic Unix socket server

**Key Files:**

- `Cargo.toml` (workspace)
- `crates/undertone-daemon/src/main.rs`
- `crates/undertone-pipewire/src/lib.rs`
- `crates/undertone-db/src/schema.rs`

---

### ✅ Milestone 2: Virtual Channels (COMPLETE)

**Deliverables:**

- [x] Create 5 virtual channel sinks via PipeWire
- [x] Create 2 mix nodes (stream-mix, monitor-mix)
- [x] Detect `wave3-source`/`wave3-sink` nodes
- [x] Monitor graph for node/port/link changes
- [x] Detect audio clients (e.g., Spotify)

**Success Criteria:**

- [x] `pw-cli list-objects Node | grep ut-ch-` shows 5 channels
- [x] `pw-cli list-objects Node | grep ut-stream-mix` shows mix node
- [x] Wave:3 detection logs on daemon startup

**Technical Implementation:**

- Uses `MainLoopRc`/`ContextRc`/`CoreRc` for proper pipewire-rs lifecycle
- `pipewire::channel` for cross-thread communication with main loop
- Stores node/link proxies in `Rc<RefCell<Vec<...>>>` to prevent destruction
- `GraphManager` caches PipeWire graph state with `parking_lot::RwLock`

---

### ✅ Milestone 3: Mix Routing (COMPLETE)

**Deliverables:**

- [x] Link channel sinks to stream-mix and monitor-mix
- [ ] Per-channel volume/mute for each mix (deferred - requires filter nodes)
- [ ] Mic input routing to mixes (deferred)
- [x] Link management (create/destroy links)

**Success Criteria:**

- [x] Audio from channel sinks flows to mix nodes
- [x] OBS can capture `ut-stream-mix` monitor ports

**Technical Implementation:**

- Added public `create_link()`, `create_stereo_links()`, `destroy_link()` methods to `PipeWireRuntime`
- Added `create_channel_to_mix_links()` helper that links all channels to both mixes
- Added `link_monitor_to_headphones()` to route monitor-mix to Wave:3 sink
- Port naming: PipeWire uses `monitor_FL`/`monitor_FR` for output, `playback_FL`/`playback_FR` for input
- Links created: 20 individual links (5 channels × 2 mixes × 2 channels)
- Added port query helpers to `GraphManager`: `get_port_by_name()`, `get_input_ports()`, `get_output_ports()`, etc.
- Links tracked in `GraphManager.created_links` for state management

**Remaining Work (Milestone 3b):**

- Per-channel volume control requires filter nodes between channels and mixes
- Mic input routing (wave3-source → mixes) with independent volume control

---

### ✅ Milestone 4: IPC Protocol (COMPLETE)

**Deliverables:**

- [x] Complete JSON protocol implementation
- [x] All request/response methods
- [x] Event subscription and broadcasting
- [x] Command-based state mutation architecture

**Success Criteria:**

- [x] IPC socket created at `$XDG_RUNTIME_DIR/undertone/daemon.sock`
- [x] GetState/GetChannels/GetDeviceStatus requests work
- [x] SetChannelVolume/SetChannelMute update in-memory state
- [x] SetAppRoute/RemoveAppRoute persist to database
- [x] Events broadcast on state changes (ChannelVolumeChanged, DeviceConnected, etc.)
- [x] Shutdown command gracefully stops daemon

**Technical Implementation:**

- Added `Command` enum in `undertone-core/src/command.rs` for state mutations
- Request handlers return `HandleResult` with response + optional command
- Commands processed in main loop with mutable access to state
- Events broadcast via `tokio::sync::broadcast` channel
- Routing rules persisted to SQLite with `RouteRule` type
- Volume/mute changes tracked in `ChannelState` (PipeWire integration deferred to Milestone 3b)

---

### ✅ Milestone 5: UI Framework (COMPLETE)

**Deliverables:**

- [x] Main window with tabs (Mixer, Apps, Device)
- [x] Mixer page with channel strips
- [ ] VU meters with real-time levels (deferred - needs PipeWire level monitoring)
- [x] Volume faders and mute buttons
- [x] cxx-qt bridge to expose Rust controller to QML

**Framework:** cxx-qt 0.7 (Qt6/QML with Rust backend)

**Success Criteria:**

- [x] UI builds and launches successfully
- [x] QML loads from compiled QRC resources
- [x] UndertoneController exposed to QML with properties
- [x] Dark theme with Wave:3 inspired color scheme

**Technical Implementation:**

- Uses `QGuiApplication` and `QQmlApplicationEngine` from cxx-qt-lib
- Bridge in `src/bridge.rs` exposes `UndertoneController` QObject to QML
- Index-based channel access (channelName, channelVolume, channelMuted, etc.)
- QML files: main.qml (app window), MixerPage.qml, ChannelStrip.qml
- Qt modules: Core, Qml, Quick, QuickControls2

---

### ⏳ Milestone 6: App Routing UI (NOT STARTED)

**Deliverables:**

- [ ] List of active audio apps
- [ ] Click-to-assign channel routing
- [ ] Persistent route rules

---

### ⏳ Milestone 7: Device Panel (NOT STARTED)

**Deliverables:**

- [ ] Device status display
- [ ] Mic gain slider (via ALSA fallback)
- [ ] Mute toggle

**Current State:**

- ALSA fallback skeleton in `undertone-hid/src/alsa_fallback.rs`
- Uses `amixer` subprocess (ALSA crate removed due to dependency issues)

---

### ⏳ Milestone 8: Profiles (NOT STARTED)

**Deliverables:**

- [ ] Save/load mixer configurations
- [ ] Default profile on startup
- [ ] Import/export as JSON

**Current State:**

- Profile types defined in `undertone-core/src/profile.rs`
- Database tables exist for persistence

---

### 🔮 Milestone 9: Wave:3 Hardware (DEFERRED)

**Status:** Deferred until core mixer is stable. Using ALSA fallback.

**Deliverables (when prioritized):**

- [ ] USB traffic capture on Windows to reverse-engineer protocol
- [ ] HID protocol implementation for Interface 3 (vendor-specific)
- [ ] Bidirectional mute sync (hardware button ↔ software state)
- [ ] LED control (if feasible)

---

### ⏳ Milestone 10: Polish & Release (NOT STARTED)

**Deliverables:**

- [ ] Diagnostics page
- [ ] Error recovery
- [ ] Installation scripts
- [ ] Documentation

---

## PipeWire Graph Topology

### Undertone Nodes (Created by Daemon)

| Node Name        | media.class | Purpose                       |
| ---------------- | ----------- | ----------------------------- |
| `ut-ch-system`   | Audio/Sink  | System sounds channel         |
| `ut-ch-voice`    | Audio/Sink  | Voice chat (Discord, Zoom)    |
| `ut-ch-music`    | Audio/Sink  | Music apps (Spotify)          |
| `ut-ch-browser`  | Audio/Sink  | Browser audio                 |
| `ut-ch-game`     | Audio/Sink  | Games                         |
| `ut-stream-mix`  | Audio/Sink  | Combined output for streaming |
| `ut-monitor-mix` | Audio/Sink  | Local monitoring mix          |

### Hardware Nodes (from WirePlumber config)

| Node Name         | Purpose                   |
| ----------------- | ------------------------- |
| `wave3-source`    | Physical mic input        |
| `wave3-sink`      | Physical headphone output |
| `wave3-null-sink` | Keeps source active       |

---

## Key Dependencies

```toml
[workspace.dependencies]
tokio = { version = "1.42", features = ["rt-multi-thread", "macros", "sync", "signal", "net", "io-util", "fs", "time"] }
pipewire = "0.9"
libspa = "0.9"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rusqlite = { version = "0.32", features = ["bundled"] }
cxx-qt = "0.7"
cxx-qt-lib = "0.7"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
thiserror = "2.0"
anyhow = "1.0"
parking_lot = "0.12"
```

**System Dependencies (Fedora):**

```bash
sudo dnf install pipewire-devel qt6-qtbase-devel qt6-qtdeclarative-devel
```

---

## Installation Paths

| Component   | Path                                                         |
| ----------- | ------------------------------------------------------------ |
| Binaries    | `~/.local/bin/undertone-daemon`, `~/.local/bin/undertone`    |
| Data        | `~/.local/share/undertone/`                                  |
| Config      | `~/.config/undertone/config.toml`                            |
| WirePlumber | `~/.config/wireplumber/wireplumber.conf.d/60-undertone.conf` |
| systemd     | `~/.config/systemd/user/undertone.service`                   |
| udev        | `/etc/udev/rules.d/70-elgato-wave3.rules`                    |

---

## Running the Daemon

```bash
# Build
cargo build -p undertone-daemon

# Run (creates database, connects to PipeWire, creates virtual sinks)
cargo run -p undertone-daemon

# Verify nodes were created
pw-cli list-objects Node | grep ut-
```

---

## Next Steps

1. **Milestone 5: UI Framework**
   - Set up cxx-qt build properly
   - Create main window with mixer page
   - Implement channel strip component
   - Connect UI to IPC for real-time updates

2. **Milestone 3b: Volume Control** (optional enhancement)
   - Add filter nodes for per-channel, per-mix volume control
   - Route mic input to mixes with volume control
   - Apply volume changes to PipeWire nodes

3. **Milestone 6: App Routing UI**
   - Display list of active audio apps
   - Implement click-to-assign channel routing
   - Show route rules with edit/delete

---

## Known Issues

1. **Qt UI disabled**: Build skipped until Qt development packages installed properly
2. **HID integration deferred**: Using ALSA fallback for mic gain control
3. **Unused warnings**: Several unused imports in scaffolding code (will be used in later milestones)

---

## Git History

```
1ac61ae docs: Add PROGRESS.md with development status
f8d6ec8 feat: Implement Undertone daemon with PipeWire integration
```
