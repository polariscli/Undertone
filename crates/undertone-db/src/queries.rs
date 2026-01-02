//! Database query functions.

use rusqlite::params;
use undertone_core::{
    channel::{ChannelConfig, ChannelState},
    mixer::MixerState,
    profile::{Profile, ProfileChannel, ProfileSummary},
    routing::{PatternType, RouteRule},
};

use crate::{Database, DbResult};

impl Database {
    /// Load all channels with their current state.
    pub fn load_channels(&self) -> DbResult<Vec<ChannelState>> {
        let mut stmt = self.conn.prepare(
            r"SELECT c.id, c.name, c.display_name, c.icon, c.color, c.sort_order, c.is_system,
                     cs.stream_volume, cs.stream_muted, cs.monitor_volume, cs.monitor_muted
              FROM channels c
              LEFT JOIN channel_state cs ON c.id = cs.channel_id
              ORDER BY c.sort_order",
        )?;

        let channels = stmt
            .query_map([], |row| {
                Ok(ChannelState {
                    config: ChannelConfig {
                        name: row.get(1)?,
                        display_name: row.get(2)?,
                        icon: row.get(3)?,
                        color: row.get(4)?,
                        sort_order: row.get(5)?,
                        is_system: row.get(6)?,
                    },
                    stream_volume: row.get::<_, Option<f64>>(7)?.unwrap_or(1.0) as f32,
                    stream_muted: row.get::<_, Option<bool>>(8)?.unwrap_or(false),
                    monitor_volume: row.get::<_, Option<f64>>(9)?.unwrap_or(1.0) as f32,
                    monitor_muted: row.get::<_, Option<bool>>(10)?.unwrap_or(false),
                    level_left: 0.0,
                    level_right: 0.0,
                    node_id: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(channels)
    }

    /// Save channel state.
    pub fn save_channel_state(&self, channel_name: &str, state: &ChannelState) -> DbResult<()> {
        self.conn.execute(
            r"UPDATE channel_state SET
                stream_volume = ?,
                stream_muted = ?,
                monitor_volume = ?,
                monitor_muted = ?,
                updated_at = datetime('now')
              WHERE channel_id = (SELECT id FROM channels WHERE name = ?)",
            params![
                state.stream_volume as f64,
                state.stream_muted,
                state.monitor_volume as f64,
                state.monitor_muted,
                channel_name,
            ],
        )?;
        Ok(())
    }

    /// Load all routing rules.
    pub fn load_routes(&self) -> DbResult<Vec<RouteRule>> {
        let mut stmt = self.conn.prepare(
            r"SELECT ar.pattern, ar.pattern_type, c.name, ar.priority
              FROM app_routes ar
              JOIN channels c ON ar.channel_id = c.id
              ORDER BY ar.priority DESC",
        )?;

        let routes = stmt
            .query_map([], |row| {
                let pattern_type_str: String = row.get(1)?;
                let pattern_type = match pattern_type_str.as_str() {
                    "exact" => PatternType::Exact,
                    "prefix" => PatternType::Prefix,
                    "regex" => PatternType::Regex,
                    _ => PatternType::Exact,
                };

                Ok(RouteRule {
                    pattern: row.get(0)?,
                    pattern_type,
                    channel: row.get(2)?,
                    priority: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(routes)
    }

    /// Add or update a routing rule.
    pub fn save_route(&self, rule: &RouteRule) -> DbResult<()> {
        let pattern_type = match rule.pattern_type {
            PatternType::Exact => "exact",
            PatternType::Prefix => "prefix",
            PatternType::Regex => "regex",
        };

        self.conn.execute(
            r"INSERT INTO app_routes (pattern, pattern_type, channel_id, priority)
              VALUES (?, ?, (SELECT id FROM channels WHERE name = ?), ?)
              ON CONFLICT(pattern) DO UPDATE SET
                pattern_type = excluded.pattern_type,
                channel_id = excluded.channel_id,
                priority = excluded.priority",
            params![rule.pattern, pattern_type, rule.channel, rule.priority],
        )?;
        Ok(())
    }

    /// Delete a routing rule.
    pub fn delete_route(&self, pattern: &str) -> DbResult<()> {
        self.conn.execute("DELETE FROM app_routes WHERE pattern = ?", params![pattern])?;
        Ok(())
    }

    /// Log an event to the database.
    pub fn log_event(
        &self,
        level: &str,
        source: &str,
        message: &str,
        data: Option<&str>,
    ) -> DbResult<()> {
        self.conn.execute(
            "INSERT INTO event_log (level, source, message, data) VALUES (?, ?, ?, ?)",
            params![level, source, message, data],
        )?;
        Ok(())
    }

    /// List all profiles.
    pub fn list_profiles(&self) -> DbResult<Vec<ProfileSummary>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, is_default, description FROM profiles ORDER BY name")?;

        let profiles = stmt
            .query_map([], |row| {
                Ok(ProfileSummary {
                    name: row.get(0)?,
                    is_default: row.get(1)?,
                    description: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(profiles)
    }

    /// Save a profile (insert or update).
    pub fn save_profile(&self, profile: &Profile) -> DbResult<()> {
        // Serialize mixer state to JSON
        let mixer_json = serde_json::to_string(&profile.mixer).map_err(|e| {
            crate::error::DbError::Serialization(format!("Failed to serialize mixer state: {e}"))
        })?;

        // Insert or update profile
        self.conn.execute(
            r"INSERT INTO profiles (name, description, is_default, mixer_state, updated_at)
              VALUES (?, ?, ?, ?, datetime('now'))
              ON CONFLICT(name) DO UPDATE SET
                description = excluded.description,
                is_default = excluded.is_default,
                mixer_state = excluded.mixer_state,
                updated_at = datetime('now')",
            params![profile.name, profile.description, profile.is_default, mixer_json,],
        )?;

        // Get profile ID
        let profile_id: i64 = self.conn.query_row(
            "SELECT id FROM profiles WHERE name = ?",
            params![profile.name],
            |row| row.get(0),
        )?;

        // Clear existing channel states for this profile
        self.conn
            .execute("DELETE FROM profile_channels WHERE profile_id = ?", params![profile_id])?;

        // Insert channel states
        for channel in &profile.channels {
            // Get channel ID
            let channel_id: Option<i64> = self
                .conn
                .query_row("SELECT id FROM channels WHERE name = ?", params![channel.name], |row| {
                    row.get(0)
                })
                .ok();

            if let Some(ch_id) = channel_id {
                self.conn.execute(
                    r"INSERT INTO profile_channels
                      (profile_id, channel_id, stream_volume, stream_muted, monitor_volume, monitor_muted)
                      VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        profile_id,
                        ch_id,
                        channel.stream_volume as f64,
                        channel.stream_muted,
                        channel.monitor_volume as f64,
                        channel.monitor_muted,
                    ],
                )?;
            }
        }

        // Clear existing routes for this profile
        self.conn
            .execute("DELETE FROM profile_routes WHERE profile_id = ?", params![profile_id])?;

        // Insert routes
        for route in &profile.routes {
            let pattern_type = match route.pattern_type {
                PatternType::Exact => "exact",
                PatternType::Prefix => "prefix",
                PatternType::Regex => "regex",
            };

            // Get channel ID
            let channel_id: Option<i64> = self
                .conn
                .query_row(
                    "SELECT id FROM channels WHERE name = ?",
                    params![route.channel],
                    |row| row.get(0),
                )
                .ok();

            if let Some(ch_id) = channel_id {
                self.conn.execute(
                    r"INSERT INTO profile_routes
                      (profile_id, pattern, pattern_type, channel_id, priority)
                      VALUES (?, ?, ?, ?, ?)",
                    params![profile_id, route.pattern, pattern_type, ch_id, route.priority,],
                )?;
            }
        }

        Ok(())
    }

    /// Load a profile by name.
    pub fn load_profile(&self, name: &str) -> DbResult<Option<Profile>> {
        // Get profile metadata
        let profile_row: Option<(i64, String, Option<String>, bool, Option<String>)> = self.conn.query_row(
            "SELECT id, name, description, is_default, mixer_state FROM profiles WHERE name = ?",
            params![name],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        ).ok();

        let Some((profile_id, profile_name, description, is_default, mixer_json)) = profile_row
        else {
            return Ok(None);
        };

        // Parse mixer state
        let mixer: MixerState =
            mixer_json.and_then(|json| serde_json::from_str(&json).ok()).unwrap_or_default();

        // Load channel states
        let mut stmt = self.conn.prepare(
            r"SELECT c.name, pc.stream_volume, pc.stream_muted, pc.monitor_volume, pc.monitor_muted
              FROM profile_channels pc
              JOIN channels c ON pc.channel_id = c.id
              WHERE pc.profile_id = ?",
        )?;

        let channels: Vec<ProfileChannel> = stmt
            .query_map(params![profile_id], |row| {
                Ok(ProfileChannel {
                    name: row.get(0)?,
                    stream_volume: row.get::<_, f64>(1)? as f32,
                    stream_muted: row.get(2)?,
                    monitor_volume: row.get::<_, f64>(3)? as f32,
                    monitor_muted: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Load routes
        let mut stmt = self.conn.prepare(
            r"SELECT pr.pattern, pr.pattern_type, c.name, pr.priority
              FROM profile_routes pr
              JOIN channels c ON pr.channel_id = c.id
              WHERE pr.profile_id = ?
              ORDER BY pr.priority DESC",
        )?;

        let routes: Vec<RouteRule> = stmt
            .query_map(params![profile_id], |row| {
                let pattern_type_str: String = row.get(1)?;
                let pattern_type = match pattern_type_str.as_str() {
                    "exact" => PatternType::Exact,
                    "prefix" => PatternType::Prefix,
                    "regex" => PatternType::Regex,
                    _ => PatternType::Exact,
                };

                Ok(RouteRule {
                    pattern: row.get(0)?,
                    pattern_type,
                    channel: row.get(2)?,
                    priority: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Some(Profile { name: profile_name, description, is_default, channels, routes, mixer }))
    }

    /// Delete a profile by name.
    pub fn delete_profile(&self, name: &str) -> DbResult<bool> {
        // Don't allow deleting the default profile
        let is_default: bool = self
            .conn
            .query_row("SELECT is_default FROM profiles WHERE name = ?", params![name], |row| {
                row.get(0)
            })
            .unwrap_or(false);

        if is_default {
            return Ok(false);
        }

        let deleted = self
            .conn
            .execute("DELETE FROM profiles WHERE name = ? AND is_default = FALSE", params![name])?;

        Ok(deleted > 0)
    }

    /// Get the default profile name.
    pub fn get_default_profile(&self) -> DbResult<Option<String>> {
        let name: Option<String> = self
            .conn
            .query_row("SELECT name FROM profiles WHERE is_default = TRUE LIMIT 1", [], |row| {
                row.get(0)
            })
            .ok();

        Ok(name)
    }
}
