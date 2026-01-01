//! Database query functions.

use rusqlite::{Row, params};
use undertone_core::{
    channel::{ChannelConfig, ChannelState},
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
}
