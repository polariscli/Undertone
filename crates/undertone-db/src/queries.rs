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

                Ok(RouteRule::new(
                    row.get(0)?,
                    pattern_type,
                    row.get(2)?,
                    row.get(3)?,
                ))
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

                Ok(RouteRule::new(
                    row.get(0)?,
                    pattern_type,
                    row.get(2)?,
                    row.get(3)?,
                ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;
    use undertone_core::mixer::MixerState;

    fn test_db() -> Database {
        Database::open_in_memory().expect("Failed to create test database")
    }

    #[test]
    fn test_load_channels_returns_default_channels() {
        let db = test_db();
        let channels = db.load_channels().expect("Failed to load channels");

        // Should have 5 default channels
        assert_eq!(channels.len(), 5);

        // Check channel names
        let names: Vec<_> = channels.iter().map(|c| c.config.name.as_str()).collect();
        assert!(names.contains(&"system"));
        assert!(names.contains(&"voice"));
        assert!(names.contains(&"music"));
        assert!(names.contains(&"browser"));
        assert!(names.contains(&"game"));
    }

    #[test]
    fn test_save_and_load_channel_state() {
        let db = test_db();

        // Load initial channels
        let channels = db.load_channels().expect("Failed to load channels");
        let music_channel = channels.iter().find(|c| c.config.name == "music").unwrap();

        // Modify and save
        let mut modified = music_channel.clone();
        modified.stream_volume = 0.5;
        modified.stream_muted = true;
        modified.monitor_volume = 0.75;
        modified.monitor_muted = false;

        db.save_channel_state("music", &modified).expect("Failed to save channel state");

        // Reload and verify
        let channels = db.load_channels().expect("Failed to reload channels");
        let music_channel = channels.iter().find(|c| c.config.name == "music").unwrap();

        assert!((music_channel.stream_volume - 0.5).abs() < 0.01);
        assert!(music_channel.stream_muted);
        assert!((music_channel.monitor_volume - 0.75).abs() < 0.01);
        assert!(!music_channel.monitor_muted);
    }

    #[test]
    fn test_load_routes_returns_default_routes() {
        let db = test_db();
        let routes = db.load_routes().expect("Failed to load routes");

        // Should have default routes
        assert!(!routes.is_empty());

        // Check that some known routes exist
        let discord_route = routes.iter().find(|r| r.pattern == "discord");
        assert!(discord_route.is_some());
        assert_eq!(discord_route.unwrap().channel, "voice");
    }

    #[test]
    fn test_save_and_delete_route() {
        let db = test_db();

        // Create a new route
        let rule = RouteRule::new("my-app".into(), PatternType::Exact, "music".into(), 200);
        db.save_route(&rule).expect("Failed to save route");

        // Verify it exists
        let routes = db.load_routes().expect("Failed to load routes");
        let my_route = routes.iter().find(|r| r.pattern == "my-app");
        assert!(my_route.is_some());
        assert_eq!(my_route.unwrap().priority, 200);

        // Delete the route
        db.delete_route("my-app").expect("Failed to delete route");

        // Verify it's gone
        let routes = db.load_routes().expect("Failed to load routes");
        let my_route = routes.iter().find(|r| r.pattern == "my-app");
        assert!(my_route.is_none());
    }

    #[test]
    fn test_save_route_upsert() {
        let db = test_db();

        // Create a route
        let rule = RouteRule::new("test-app".into(), PatternType::Exact, "music".into(), 100);
        db.save_route(&rule).expect("Failed to save route");

        // Update the same route with different values
        let updated_rule = RouteRule::new("test-app".into(), PatternType::Prefix, "voice".into(), 150);
        db.save_route(&updated_rule).expect("Failed to update route");

        // Verify the update
        let routes = db.load_routes().expect("Failed to load routes");
        let test_route = routes.iter().find(|r| r.pattern == "test-app").unwrap();

        assert_eq!(test_route.pattern_type, PatternType::Prefix);
        assert_eq!(test_route.channel, "voice");
        assert_eq!(test_route.priority, 150);
    }

    #[test]
    fn test_list_profiles_has_default() {
        let db = test_db();
        let profiles = db.list_profiles().expect("Failed to list profiles");

        // Should have the default profile
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Default");
        assert!(profiles[0].is_default);
    }

    #[test]
    fn test_save_and_load_profile() {
        let db = test_db();

        // Create a profile
        let profile = Profile {
            name: "test-profile".into(),
            description: Some("A test profile".into()),
            is_default: false,
            channels: vec![ProfileChannel {
                name: "music".into(),
                stream_volume: 0.8,
                stream_muted: false,
                monitor_volume: 0.6,
                monitor_muted: true,
            }],
            routes: vec![RouteRule::new("custom-app".into(), PatternType::Exact, "music".into(), 100)],
            mixer: MixerState::default(),
        };

        db.save_profile(&profile).expect("Failed to save profile");

        // Load it back
        let loaded = db.load_profile("test-profile").expect("Failed to load profile");
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "test-profile");
        assert_eq!(loaded.description, Some("A test profile".into()));
        assert!(!loaded.is_default);
        assert_eq!(loaded.channels.len(), 1);
        assert_eq!(loaded.routes.len(), 1);
    }

    #[test]
    fn test_profile_not_found() {
        let db = test_db();
        let loaded = db.load_profile("nonexistent").expect("Failed to query profile");
        assert!(loaded.is_none());
    }

    #[test]
    fn test_delete_profile() {
        let db = test_db();

        // Create a profile
        let profile = Profile {
            name: "deleteme".into(),
            description: None,
            is_default: false,
            channels: vec![],
            routes: vec![],
            mixer: MixerState::default(),
        };
        db.save_profile(&profile).expect("Failed to save profile");

        // Verify it exists
        let profiles = db.list_profiles().expect("Failed to list profiles");
        assert!(profiles.iter().any(|p| p.name == "deleteme"));

        // Delete it
        let deleted = db.delete_profile("deleteme").expect("Failed to delete profile");
        assert!(deleted);

        // Verify it's gone
        let profiles = db.list_profiles().expect("Failed to list profiles");
        assert!(!profiles.iter().any(|p| p.name == "deleteme"));
    }

    #[test]
    fn test_cannot_delete_default_profile() {
        let db = test_db();

        // Create a default profile
        let profile = Profile {
            name: "default".into(),
            description: None,
            is_default: true,
            channels: vec![],
            routes: vec![],
            mixer: MixerState::default(),
        };
        db.save_profile(&profile).expect("Failed to save profile");

        // Try to delete it
        let deleted = db.delete_profile("default").expect("Failed to attempt delete");
        assert!(!deleted); // Should return false

        // Verify it still exists
        let loaded = db.load_profile("default").expect("Failed to load profile");
        assert!(loaded.is_some());
    }

    #[test]
    fn test_log_event() {
        let db = test_db();

        // Should not fail
        db.log_event("info", "test", "Test message", Some(r#"{"key": "value"}"#))
            .expect("Failed to log event");

        db.log_event("error", "test", "Error message", None)
            .expect("Failed to log event without data");
    }

    #[test]
    fn test_route_pattern_types() {
        let db = test_db();

        // Test all pattern types
        let exact = RouteRule::new("exact-app".into(), PatternType::Exact, "music".into(), 100);
        let prefix = RouteRule::new("prefix-app".into(), PatternType::Prefix, "voice".into(), 100);
        let regex = RouteRule::new(r"regex-\d+".into(), PatternType::Regex, "game".into(), 100);

        db.save_route(&exact).expect("Failed to save exact route");
        db.save_route(&prefix).expect("Failed to save prefix route");
        db.save_route(&regex).expect("Failed to save regex route");

        let routes = db.load_routes().expect("Failed to load routes");

        let exact_loaded = routes.iter().find(|r| r.pattern == "exact-app").unwrap();
        let prefix_loaded = routes.iter().find(|r| r.pattern == "prefix-app").unwrap();
        let regex_loaded = routes.iter().find(|r| r.pattern == r"regex-\d+").unwrap();

        assert_eq!(exact_loaded.pattern_type, PatternType::Exact);
        assert_eq!(prefix_loaded.pattern_type, PatternType::Prefix);
        assert_eq!(regex_loaded.pattern_type, PatternType::Regex);
    }
}
