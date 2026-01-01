//! Database schema definition.

/// Initial schema (version 1).
pub const SCHEMA_V1: &str = r#"
-- Schema version tracking
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Channel definitions
CREATE TABLE IF NOT EXISTS channels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    icon TEXT,
    color TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Channel volumes/mutes (current state)
CREATE TABLE IF NOT EXISTS channel_state (
    channel_id INTEGER PRIMARY KEY REFERENCES channels(id) ON DELETE CASCADE,
    stream_volume REAL NOT NULL DEFAULT 1.0,
    stream_muted BOOLEAN NOT NULL DEFAULT FALSE,
    monitor_volume REAL NOT NULL DEFAULT 1.0,
    monitor_muted BOOLEAN NOT NULL DEFAULT FALSE,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- App routing rules
CREATE TABLE IF NOT EXISTS app_routes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL UNIQUE,
    pattern_type TEXT NOT NULL DEFAULT 'exact',
    channel_id INTEGER NOT NULL REFERENCES channels(id),
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Saved profiles
CREATE TABLE IF NOT EXISTS profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Profile channel states
CREATE TABLE IF NOT EXISTS profile_channels (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    channel_id INTEGER NOT NULL REFERENCES channels(id),
    stream_volume REAL NOT NULL,
    stream_muted BOOLEAN NOT NULL,
    monitor_volume REAL NOT NULL,
    monitor_muted BOOLEAN NOT NULL,
    PRIMARY KEY (profile_id, channel_id)
);

-- Profile app routes
CREATE TABLE IF NOT EXISTS profile_routes (
    profile_id INTEGER NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    pattern TEXT NOT NULL,
    pattern_type TEXT NOT NULL,
    channel_id INTEGER NOT NULL REFERENCES channels(id),
    priority INTEGER NOT NULL,
    PRIMARY KEY (profile_id, pattern)
);

-- Device settings
CREATE TABLE IF NOT EXISTS device_settings (
    device_serial TEXT PRIMARY KEY,
    mic_gain REAL NOT NULL DEFAULT 0.5,
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Event log for diagnostics
CREATE TABLE IF NOT EXISTS event_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    level TEXT NOT NULL,
    source TEXT NOT NULL,
    message TEXT NOT NULL,
    data TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_app_routes_pattern ON app_routes(pattern);
CREATE INDEX IF NOT EXISTS idx_event_log_timestamp ON event_log(timestamp);
CREATE INDEX IF NOT EXISTS idx_profiles_default ON profiles(is_default);
"#;

/// Default data to insert after schema creation.
pub const DEFAULT_DATA: &str = r#"
-- Default channels
INSERT OR IGNORE INTO channels (name, display_name, icon, sort_order, is_system) VALUES
    ('system', 'System', 'audio-volume-high', 0, TRUE),
    ('voice', 'Voice', 'microphone', 1, TRUE),
    ('music', 'Music', 'audio-headphones', 2, TRUE),
    ('browser', 'Browser', 'web-browser', 3, TRUE),
    ('game', 'Game', 'applications-games', 4, TRUE);

-- Initialize channel states
INSERT OR IGNORE INTO channel_state (channel_id, stream_volume, monitor_volume)
    SELECT id, 1.0, 1.0 FROM channels;

-- Default app routes
INSERT OR IGNORE INTO app_routes (pattern, pattern_type, channel_id, priority) VALUES
    ('discord', 'prefix', (SELECT id FROM channels WHERE name = 'voice'), 100),
    ('zoom', 'prefix', (SELECT id FROM channels WHERE name = 'voice'), 100),
    ('teams', 'prefix', (SELECT id FROM channels WHERE name = 'voice'), 100),
    ('spotify', 'exact', (SELECT id FROM channels WHERE name = 'music'), 100),
    ('rhythmbox', 'exact', (SELECT id FROM channels WHERE name = 'music'), 100),
    ('firefox', 'exact', (SELECT id FROM channels WHERE name = 'browser'), 50),
    ('chromium', 'prefix', (SELECT id FROM channels WHERE name = 'browser'), 50),
    ('chrome', 'prefix', (SELECT id FROM channels WHERE name = 'browser'), 50),
    ('steam', 'exact', (SELECT id FROM channels WHERE name = 'game'), 100);

-- Default profile
INSERT OR IGNORE INTO profiles (name, description, is_default) VALUES
    ('Default', 'Default mixer configuration', TRUE);
"#;
