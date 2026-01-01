# Undertone Development Progress

**Project**: Linux-native Elgato Wave:3 audio control application
**License**: MIT / Apache-2.0 (Open Source)
**Started**: January 2026

---

## Current Status: Milestone 2 Complete

The daemon successfully connects to PipeWire, creates virtual audio channels, and detects Wave:3 hardware.

### Verified Working Features

```bash
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

### ⏳ Milestone 3: Mix Routing (NOT STARTED)

**Deliverables:**

- [ ] Link channel sinks to stream-mix and monitor-mix
- [ ] Per-channel volume/mute for each mix
- [ ] Mic input routing to mixes
- [ ] Link management (create/destroy links)

**Success Criteria:**

- [ ] Audio from channel sinks flows to mix nodes
- [ ] OBS can capture `ut-stream-mix` monitor ports

**Implementation Notes:**

- Need to implement `create_link` in runtime.rs (infrastructure exists)
- Volume control via PipeWire node properties or filter nodes
- May need `filter-chain` or `loopback` modules for mixing

---

### ⏳ Milestone 4: IPC Protocol (NOT STARTED)

**Deliverables:**

- [ ] Complete JSON protocol implementation
- [ ] All request/response methods
- [ ] Event subscription and broadcasting
- [ ] Client library for UI

**Current State:**

- Server skeleton exists in `undertone-ipc/src/server.rs`
- Message types defined in `undertone-ipc/src/messages.rs`
- Event types defined in `undertone-ipc/src/events.rs`

---

### ⏳ Milestone 5: UI Framework (NOT STARTED)

**Deliverables:**

- [ ] Main window with tabs
- [ ] Mixer page with channel strips
- [ ] VU meters with real-time levels
- [ ] Volume faders and mute buttons

**Framework:** cxx-qt (Qt6/QML with Rust backend)

**Current State:**

- Skeleton crate exists in `undertone-ui/`
- Build disabled until Qt development packages installed
- State management pattern established

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

1. **Milestone 3: Mix Routing**
   - Implement link creation between channel sinks and mix nodes
   - Add volume control via PipeWire properties
   - Route mic input to appropriate mixes

2. **Milestone 4: IPC Protocol**
   - Implement request handlers in daemon
   - Add event broadcasting for state changes
   - Create client library for UI

3. **Milestone 5: UI Framework**
   - Set up cxx-qt build properly
   - Create main window with mixer page
   - Implement channel strip component

---

## Known Issues

1. **Qt UI disabled**: Build skipped until Qt development packages installed properly
2. **HID integration deferred**: Using ALSA fallback for mic gain control
3. **Unused warnings**: Several unused imports in scaffolding code (will be used in later milestones)

---

## Git History

```
f8d6ec8 feat: Implement Undertone daemon with PipeWire integration
```
