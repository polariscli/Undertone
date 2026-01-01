//! Application routing rules and matching.

use serde::{Deserialize, Serialize};

/// Pattern type for matching applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternType {
    /// Exact match on application name
    Exact,
    /// Prefix match (app name starts with pattern)
    Prefix,
    /// Regular expression match
    Regex,
}

/// A rule for routing applications to channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    /// Pattern to match against application name
    pub pattern: String,
    /// Type of pattern matching
    pub pattern_type: PatternType,
    /// Target channel name
    pub channel: String,
    /// Priority (higher = matched first)
    pub priority: i32,
}

impl RouteRule {
    /// Check if this rule matches an application name.
    #[must_use]
    pub fn matches(&self, app_name: &str) -> bool {
        match self.pattern_type {
            PatternType::Exact => app_name == self.pattern,
            PatternType::Prefix => app_name.starts_with(&self.pattern),
            PatternType::Regex => {
                // TODO: Compile and cache regex
                regex::Regex::new(&self.pattern).map(|re| re.is_match(app_name)).unwrap_or(false)
            }
        }
    }
}

/// An active application route.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRoute {
    /// PipeWire client/node ID
    pub app_id: u32,
    /// Application name (from PipeWire properties)
    pub app_name: String,
    /// Binary name (for persistent routing)
    pub binary_name: Option<String>,
    /// Process ID
    pub pid: Option<u32>,
    /// Currently assigned channel
    pub channel: String,
    /// Whether this is a saved/persistent route
    pub is_persistent: bool,
}

/// Default routing rules for common applications.
pub fn default_routes() -> Vec<RouteRule> {
    vec![
        // Voice chat applications
        RouteRule {
            pattern: "discord".to_string(),
            pattern_type: PatternType::Prefix,
            channel: "voice".to_string(),
            priority: 100,
        },
        RouteRule {
            pattern: "zoom".to_string(),
            pattern_type: PatternType::Prefix,
            channel: "voice".to_string(),
            priority: 100,
        },
        RouteRule {
            pattern: "teams".to_string(),
            pattern_type: PatternType::Prefix,
            channel: "voice".to_string(),
            priority: 100,
        },
        // Music applications
        RouteRule {
            pattern: "spotify".to_string(),
            pattern_type: PatternType::Exact,
            channel: "music".to_string(),
            priority: 100,
        },
        RouteRule {
            pattern: "rhythmbox".to_string(),
            pattern_type: PatternType::Exact,
            channel: "music".to_string(),
            priority: 100,
        },
        // Browsers
        RouteRule {
            pattern: "firefox".to_string(),
            pattern_type: PatternType::Exact,
            channel: "browser".to_string(),
            priority: 50,
        },
        RouteRule {
            pattern: "chromium".to_string(),
            pattern_type: PatternType::Prefix,
            channel: "browser".to_string(),
            priority: 50,
        },
        RouteRule {
            pattern: "chrome".to_string(),
            pattern_type: PatternType::Prefix,
            channel: "browser".to_string(),
            priority: 50,
        },
        // Games
        RouteRule {
            pattern: "steam".to_string(),
            pattern_type: PatternType::Exact,
            channel: "game".to_string(),
            priority: 100,
        },
    ]
}
