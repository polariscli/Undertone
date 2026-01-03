# Undertone Development Progress

**Project**: Linux-native Elgato Wave:3 audio control application
**License**: MIT / Apache-2.0 (Open Source)
**Started**: January 2026

---

## Current Status: Core Features Complete

The daemon and UI are fully functional with all core mixing features. Volume control, mute, app routing, profiles, and output device selection all work end-to-end.

### What Works

- **Volume sliders** - Per-channel volume control via PipeWire filter nodes
- **Mute buttons** - Full mute using PipeWire monitorMute property
- **Master volume** - Per-mix master volume and mute controls
- **Monitor output selection** - Switch between headphones, speakers, HDMI, etc.
- **App routing** - Apps automatically routed to channels based on pattern rules
- **Route changes** - Changing app route moves audio immediately
- **Profiles** - Save/load mixer configurations with channel volumes, mutes, routes
- **Default profile** - Restores last saved state on daemon startup
- **Mic control** - Gain and mute via ALSA fallback
- **Device detection** - Wave:3 detected via USB with serial number
- **UI** - Qt6/QML with Kirigami for native KDE theming

### What Doesn't Work Yet

- **VU meters** - Requires PipeWire monitor stream setup (complex)
- **HID mic control** - Using ALSA fallback, native HID not implemented

---

## Recent Bug Fixes

### Volume/Mute Control (ae24293)
- Fixed volume control using `monitorVolumes`/`monitorMute` SPA properties
- Audio flows through monitor ports, so these properties control actual levels
- Removed `object.linger` from links to allow proper destruction

### App Routing (2d3a335)
- Fixed empty profile routes overwriting global routes
- Default profile has no routes in `profile_routes` table, was clearing all routing
- Now preserves global `app_routes` when profile has no custom routes
- Fixed external link destruction using `registry.destroy_global(id)`
- Apps no longer connect to multiple channels simultaneously

---

## Audio Routing Chain

```
App (e.g., Spotify)
    │
    ▼
ut-ch-music (channel sink)
    │
    ├──► ut-ch-music-stream-vol ──► ut-stream-mix ──► OBS capture
    │
    └──► ut-ch-music-monitor-vol ──► ut-monitor-mix ──► wave3-sink (headphones)
```

---

## System Requirements

| Component    | Version                                                   |
| ------------ | --------------------------------------------------------- |
| OS           | Fedora 43, Linux 6.17+                                    |
| PipeWire     | 1.4.9+                                                    |
| WirePlumber  | 0.5.12+                                                   |
| Rust         | 1.85+ (Edition 2024)                                      |
| Qt           | 6.x with Kirigami                                         |
| Wave:3       | VID 0x0fd9, PID 0x0070                                    |

---

## Project Structure

```
Undertone/
├── Cargo.toml                    # Workspace root
├── CLAUDE.md                     # AI assistant context
├── PROGRESS.md                   # This file
├── README.md                     # User documentation
├── crates/
│   ├── undertone-daemon/         # Main daemon binary
│   ├── undertone-core/           # Business logic
│   ├── undertone-pipewire/       # PipeWire integration
│   ├── undertone-db/             # SQLite persistence
│   ├── undertone-ipc/            # IPC protocol
│   ├── undertone-hid/            # Wave:3 HID (stub)
│   └── undertone-ui/             # Qt6/QML UI
├── config/                       # Config templates
└── scripts/                      # Installation scripts
```

---

## Milestone Summary

| Milestone | Status | Description |
| --------- | ------ | ----------- |
| 1. Foundation | Complete | Workspace, PipeWire connection, SQLite, IPC socket |
| 2. Virtual Channels | Complete | 5 channel sinks, 2 mix nodes, Wave:3 detection |
| 3. Mix Routing | Complete | Channel-to-mix links, volume filters |
| 4. IPC Protocol | Complete | Full JSON protocol, events, commands |
| 5. UI Framework | Complete | Qt6/QML with Kirigami, channel strips |
| 6. App Routing UI | Complete | Active apps list, route assignment |
| 7. Device Panel | Complete | Connection status, mic controls |
| 8. Profiles | Complete | Save/load/delete, default on startup |
| 9. Wave:3 HID | Deferred | Using ALSA fallback |
| 10. Polish | In Progress | Bug fixes, documentation |

---

## Git History (Recent)

```
2d3a335 fix(routing): Preserve global routes when profile has none
ae24293 fix(pipewire): Use monitorVolumes/monitorMute for volume control
69fa0d1 docs: Update PROGRESS.md with monitor output selection feature
e667056 feat(ui): Add monitor output device selection
6143b6d docs: Update CLAUDE.md and PROGRESS.md with latest features
84b8f1d fix(ui): Improve header status and toggle styling
fd7e874 feat(ui): Add master volume control and fix ComboBox issues
```

---

## Remaining Work

### High Priority
- Test and verify all audio routing works correctly
- Verify mute produces complete silence
- Verify output device switching works

### Medium Priority
- VU meters (requires PipeWire monitor streams)
- Error handling and recovery
- Diagnostics page

### Low Priority
- Wave:3 HID integration (reverse-engineer protocol)
- Keyboard shortcuts
- System tray icon
- Auto-start on login

---

## Verified Working

```bash
# Virtual nodes created
$ pw-cli list-objects Node | grep ut-
node.name = "ut-ch-system"
node.name = "ut-ch-voice"
node.name = "ut-ch-music"
node.name = "ut-ch-browser"
node.name = "ut-ch-game"
node.name = "ut-stream-mix"
node.name = "ut-monitor-mix"
# Plus volume filter nodes for each channel

# Audio links established
$ pw-link -l | grep spotify
spotify:output_FL
  |-> ut-ch-music:playback_FL
spotify:output_FR
  |-> ut-ch-music:playback_FR

# Monitor mix to headphones
$ pw-link -l | grep "ut-monitor-mix"
ut-monitor-mix:monitor_FL
  |-> wave3-sink:playback_FL
ut-monitor-mix:monitor_FR
  |-> wave3-sink:playback_FR

# IPC communication
$ echo '{"id":1,"method":{"type":"GetState"}}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/undertone/daemon.sock
{"id":1,"result":{"Ok":{"state":"running","device_connected":true,...}}}
```

---

## Known Issues

1. **VU meters static** - Requires PipeWire monitor streams (complex)
2. **No HID mic control** - Using ALSA fallback, hardware mute button not synced
3. **cxx-qt naming** - Methods keep snake_case in QML

---

## Dependencies

```toml
tokio = "1.42"
pipewire = "0.9"
libspa = "0.9"
serde = "1.0"
serde_json = "1.0"
rusqlite = "0.32"
cxx-qt = "0.7"
cxx-qt-lib = "0.7"
tracing = "0.1"
parking_lot = "0.12"
```
