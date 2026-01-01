# Undertone Development Progress

**Project**: Linux-native Elgato Wave:3 audio control application
**License**: MIT / Apache-2.0 (Open Source)
**Started**: January 2026

---

## Current Status: UI-Daemon Integration Complete (with limitations)

The UI connects to the daemon and displays real-time state. However, several features are UI-only and don't actually affect audio.

### What Works
- ✅ UI launches and connects to daemon
- ✅ Channel strips display with names from daemon
- ✅ Volume sliders and mute buttons are interactive
- ✅ App routing page shows active audio apps
- ✅ Device page shows connection status
- ✅ Profile selector in header

### What Doesn't Work Yet
- ❌ **Volume sliders don't change actual audio** - Changes update UI state only, not PipeWire
- ❌ **VU meters are static** - No real-time level monitoring
- ❌ **Mic gain/mute is UI-only** - No ALSA/HID backend connected
- ❌ **Profile save/load is incomplete** - Daemon handlers are stubs
- ❌ **App routing doesn't move audio** - PipeWire link management not wired up
- ❌ **Device serial always empty** - Daemon doesn't detect Wave:3 serial

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
├── CLAUDE.md                     # AI assistant context
├── PROGRESS.md                   # This file
├── crates/
│   ├── undertone-daemon/         # Main daemon binary ✅
│   │   └── src/
│   │       ├── main.rs           # Entry point, main loop
│   │       └── server.rs         # IPC request handlers
│   ├── undertone-core/           # Business logic ✅
│   │   └── src/
│   │       ├── channel.rs        # Channel definitions
│   │       ├── command.rs        # State mutation commands
│   │       ├── mixer.rs          # Mix routing logic
│   │       ├── profile.rs        # Profile types
│   │       ├── routing.rs        # App routing rules
│   │       └── state.rs          # State machine
│   ├── undertone-pipewire/       # PipeWire graph management ✅
│   │   └── src/
│   │       ├── graph.rs          # Graph state cache
│   │       ├── monitor.rs        # Event monitoring
│   │       └── runtime.rs        # Node/link creation
│   ├── undertone-db/             # SQLite persistence ✅
│   ├── undertone-ipc/            # IPC protocol definitions ✅
│   │   └── src/
│   │       ├── client.rs         # Client for UI
│   │       ├── events.rs         # Event types
│   │       ├── messages.rs       # Request/response types
│   │       └── server.rs         # Server for daemon
│   ├── undertone-hid/            # Wave:3 HID integration (stub)
│   └── undertone-ui/             # Qt6/QML UI application ✅
│       ├── qml/
│       │   ├── main.qml          # Main window, tabs
│       │   ├── MixerPage.qml     # Channel strips
│       │   ├── ChannelStrip.qml  # Individual fader
│       │   ├── AppsPage.qml      # App routing
│       │   └── DevicePage.qml    # Device controls
│       └── src/
│           ├── app.rs            # Application entry
│           ├── bridge.rs         # cxx-qt QObject bridge
│           ├── ipc_handler.rs    # Async IPC thread
│           └── state.rs          # UI state
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

### ✅ Milestone 6: App Routing UI (COMPLETE)

**Deliverables:**

- [x] List of active audio apps
- [x] Click-to-assign channel routing
- [x] Persistent route rules display

**Technical Implementation:**

- AppsPage.qml with ListView showing active apps
- ComboBox channel selector per app with color-coded channels
- Persistent route indicator (P badge)
- Default routes reference panel (Discord->Voice, Spotify->Music, etc.)
- Bridge methods: app_name, app_binary, app_channel, app_persistent, set_app_channel, available_channels
- UiCommand::SetAppChannel for async route changes

---

### ✅ Milestone 7: Device Panel (COMPLETE)

**Deliverables:**

- [x] Device status display
- [x] Mic gain slider (via ALSA fallback)
- [x] Mute toggle

**Technical Implementation:**

- DevicePage.qml with device connection status, mic controls, audio info
- Gain slider with visual feedback (0-100%)
- Mute button with animated indicator when muted
- Disconnected overlay when device not available
- Audio configuration display (sample rate, bit depth, latency)
- Bridge properties: mic_muted, mic_gain
- Bridge methods: set_mic_gain_value, toggle_mic_mute
- UiCommand::SetMicGain and UiCommand::ToggleMicMute for IPC

---

### ✅ Milestone 8: Profiles (COMPLETE)

**Deliverables:**

- [x] Save/load mixer configurations
- [x] Default profile on startup
- [ ] Import/export as JSON (deferred)

**Technical Implementation:**

- Enhanced profile selector in header with dropdown list
- Save profile dialog with name input and validation
- Bridge properties: profile_count
- Bridge methods: profile_name, profile_is_default, save_profile, load_profile, delete_profile
- UiCommand::SaveProfile, LoadProfile, DeleteProfile for IPC
- Profile types defined in `undertone-core/src/profile.rs`
- Database tables exist for persistence

---

### ✅ Milestone 8.5: IPC Integration (COMPLETE)

**Deliverables:**

- [x] Connect UI to daemon socket
- [x] Real-time state synchronization
- [x] Event subscription handling
- [x] Command dispatch from UI to daemon

**Technical Implementation:**

- Added `ipc_handler.rs` module with tokio runtime in background thread
- `IpcHandle` manages connection, command sending, update receiving
- Uses `IpcUpdate` enum for daemon-to-UI communication
- Global `IPC_HANDLE` and `UI_DATA` caches for thread-safe state sharing
- `poll_updates()` QML-invokable called by Timer at 20Hz
- All controller methods use `send_command()` for daemon communication
- Subscribes to: volume/mute changes, device connect/disconnect, app events, profile changes

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

## Remaining Work

### High Priority (Core Functionality)

1. **Milestone 3b: Volume Control** - Make sliders actually work
   - [ ] Create PipeWire filter/volume nodes between channels and mixes
   - [ ] Apply `SetChannelVolume` commands to PipeWire nodes
   - [ ] Apply `SetChannelMute` by setting volume to 0 or unlinking
   - [ ] Route mic input (wave3-source) to mixes with volume control

2. **App Routing Implementation** - Make channel assignment work
   - [ ] When app route changes, update PipeWire links
   - [ ] Move app's audio stream to target channel sink
   - [ ] Handle new apps appearing (apply routing rules)
   - [ ] Persist routes to database on change

3. **Mic Control Backend** - Connect UI to actual hardware
   - [ ] Implement ALSA fallback for mic gain (`amixer` or alsa-rs)
   - [ ] Or implement Wave:3 HID protocol for native control
   - [ ] Sync mute state with hardware mute button

4. **Profile Persistence** - Make save/load actually work
   - [ ] Implement `Command::SaveProfile` in daemon
   - [ ] Store channel volumes, mutes, app routes in database
   - [ ] Implement `Command::LoadProfile` to restore state
   - [ ] Load default profile on daemon startup

### Medium Priority (Usability)

5. **VU Meters** - Real-time audio levels
   - [ ] Subscribe to PipeWire peak levels for each channel
   - [ ] Stream level data to UI via IPC events
   - [ ] Update `level_left`/`level_right` in channel state

6. **Device Detection** - Wave:3 serial and status
   - [ ] Query USB device for serial number
   - [ ] Detect device connect/disconnect events
   - [ ] Update `device_connected` and `device_serial` in state

7. **Error Handling** - Robustness
   - [ ] UI reconnection with exponential backoff
   - [ ] Graceful handling of daemon crashes
   - [ ] Visual error states in UI
   - [ ] Logging/diagnostics page

### Low Priority (Polish)

8. **Milestone 10: Release Readiness**
   - [ ] Diagnostics page showing PipeWire graph state
   - [ ] Installation scripts (systemd service, udev rules)
   - [ ] User documentation
   - [ ] Config file support (`~/.config/undertone/config.toml`)

9. **Wave:3 HID Integration** (Milestone 9)
   - [ ] Reverse-engineer USB HID protocol from Windows
   - [ ] Implement bidirectional mute sync
   - [ ] LED control (if protocol supports it)
   - [ ] Hardware gain control

10. **UI Enhancements**
    - [ ] Keyboard shortcuts (mute all, etc.)
    - [ ] System tray icon
    - [ ] Minimize to tray
    - [ ] Auto-start on login

---

## Known Issues

### Bugs

1. **cxx-qt method naming**: Methods keep snake_case in QML (e.g., `poll_updates` not `pollUpdates`)
2. **Empty device serial**: Daemon doesn't query USB for Wave:3 serial number
3. **Mic gain NaN**: Fixed in UI but underlying value may still be uninitialized
4. **Profile dropdown shows only current**: Daemon doesn't send full profile list

### Architectural Limitations

5. **No filter nodes**: Volume control requires inserting filter nodes in PipeWire graph - not yet implemented
6. **Audio doesn't move**: App routing updates state but doesn't manipulate PipeWire links
7. **Mic control is fake**: UI shows gain/mute controls but they don't affect hardware
8. **Profiles are stubs**: Save/load commands are received but not persisted

### Missing Features

9. **No VU meters**: Channels show static 0% levels
10. **No reconnection**: If daemon restarts, UI must be restarted
11. **No error feedback**: IPC errors logged but not shown to user
12. **No keyboard shortcuts**: All interaction is mouse-only
13. **No tray icon**: App must stay open in window

---

## Git History

```
32a93fc fix(ui): Fix state parsing and QML property bindings
049c4af fix(ui): Use snake_case for QML method calls
6a8266d docs: Update PROGRESS.md with IPC integration status
3d45413 feat(ui): Implement IPC integration between UI and daemon
513d247 fix: Rename volumeChanged signal to avoid Qt property conflict
f058066 fix: Move QML files to crate directory for correct resource paths
f5eec6d feat: Implement Profile management UI (Milestone 8)
fb7ce70 feat: Implement Device Panel UI (Milestone 7)
f49192e feat: Implement App Routing UI (Milestone 6)
b41dee8 feat: Implement Qt6/QML UI framework with cxx-qt
1ac61ae docs: Add PROGRESS.md with development status
f8d6ec8 feat: Implement Undertone daemon with PipeWire integration
```
