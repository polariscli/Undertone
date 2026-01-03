//! IPC message types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use undertone_core::mixer::MixType;

/// Request envelope sent from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request ID for matching responses
    pub id: u64,
    /// The method to invoke
    pub method: Method,
}

/// Response envelope sent from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID this is responding to
    pub id: u64,
    /// Result of the request
    pub result: Result<Value, ErrorInfo>,
}

/// Error information in a response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
}

impl ErrorInfo {
    /// Create a new error.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

/// Methods that can be invoked via IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Method {
    // State queries
    /// Get the current daemon state snapshot
    GetState,
    /// Get all channels
    GetChannels,
    /// Get a specific channel by name
    GetChannel { name: String },
    /// Get all active apps
    GetApps,
    /// Get all profiles
    GetProfiles,
    /// Get a specific profile by name
    GetProfile { name: String },
    /// Get device connection status
    GetDeviceStatus,
    /// Get diagnostic information
    GetDiagnostics,

    // Channel control
    /// Set volume for a channel in a specific mix
    SetChannelVolume { channel: String, mix: MixType, volume: f32 },
    /// Set mute state for a channel in a specific mix
    SetChannelMute { channel: String, mix: MixType, muted: bool },

    // Master volume control
    /// Set master volume for a mix (0.0 - 1.0)
    SetMasterVolume { mix: MixType, volume: f32 },
    /// Set master mute state for a mix
    SetMasterMute { mix: MixType, muted: bool },

    // App routing
    /// Route an app to a channel
    SetAppRoute { app_pattern: String, channel: String },
    /// Remove an app route
    RemoveAppRoute { app_pattern: String },

    // Profile management
    /// Save current state as a profile
    SaveProfile { name: String },
    /// Load a saved profile
    LoadProfile { name: String },
    /// Delete a profile
    DeleteProfile { name: String },

    // Device control
    /// Set microphone gain (0.0 - 1.0)
    SetMicGain { gain: f32 },
    /// Set microphone mute state
    SetMicMute { muted: bool },

    // Output device control
    /// Get available audio output devices
    GetOutputDevices,
    /// Set the monitor mix output device
    SetMonitorOutput { device_name: String },

    // Subscriptions
    /// Subscribe to event types
    Subscribe { events: Vec<String> },
    /// Unsubscribe from event types
    Unsubscribe { events: Vec<String> },

    // System
    /// Request graceful shutdown
    Shutdown,
    /// Force reconciliation of `PipeWire` state
    Reconcile,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip_request(request: &Request) -> Request {
        let json = serde_json::to_string(request).expect("Failed to serialize request");
        serde_json::from_str(&json).expect("Failed to deserialize request")
    }

    fn roundtrip_response(response: &Response) -> Response {
        let json = serde_json::to_string(response).expect("Failed to serialize response");
        serde_json::from_str(&json).expect("Failed to deserialize response")
    }

    #[test]
    fn test_request_get_state() {
        let request = Request { id: 1, method: Method::GetState };
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains(r#""type":"GetState""#));

        let parsed = roundtrip_request(&request);
        assert_eq!(parsed.id, 1);
        assert!(matches!(parsed.method, Method::GetState));
    }

    #[test]
    fn test_request_get_channel() {
        let request = Request { id: 2, method: Method::GetChannel { name: "music".into() } };
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains(r#""type":"GetChannel""#));
        assert!(json.contains(r#""name":"music""#));

        let parsed = roundtrip_request(&request);
        if let Method::GetChannel { name } = parsed.method {
            assert_eq!(name, "music");
        } else {
            panic!("Expected GetChannel method");
        }
    }

    #[test]
    fn test_request_set_channel_volume() {
        let request = Request {
            id: 3,
            method: Method::SetChannelVolume {
                channel: "voice".into(),
                mix: MixType::Stream,
                volume: 0.75,
            },
        };
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains(r#""type":"SetChannelVolume""#));
        assert!(json.contains(r#""channel":"voice""#));

        let parsed = roundtrip_request(&request);
        if let Method::SetChannelVolume { channel, mix, volume } = parsed.method {
            assert_eq!(channel, "voice");
            assert!(matches!(mix, MixType::Stream));
            assert!((volume - 0.75).abs() < 0.01);
        } else {
            panic!("Expected SetChannelVolume method");
        }
    }

    #[test]
    fn test_request_set_channel_mute() {
        let request = Request {
            id: 4,
            method: Method::SetChannelMute {
                channel: "music".into(),
                mix: MixType::Monitor,
                muted: true,
            },
        };

        let parsed = roundtrip_request(&request);
        if let Method::SetChannelMute { channel, mix, muted } = parsed.method {
            assert_eq!(channel, "music");
            assert!(matches!(mix, MixType::Monitor));
            assert!(muted);
        } else {
            panic!("Expected SetChannelMute method");
        }
    }

    #[test]
    fn test_request_set_app_route() {
        let request = Request {
            id: 5,
            method: Method::SetAppRoute { app_pattern: "spotify".into(), channel: "music".into() },
        };

        let parsed = roundtrip_request(&request);
        if let Method::SetAppRoute { app_pattern, channel } = parsed.method {
            assert_eq!(app_pattern, "spotify");
            assert_eq!(channel, "music");
        } else {
            panic!("Expected SetAppRoute method");
        }
    }

    #[test]
    fn test_request_save_profile() {
        let request = Request { id: 6, method: Method::SaveProfile { name: "my-profile".into() } };

        let parsed = roundtrip_request(&request);
        if let Method::SaveProfile { name } = parsed.method {
            assert_eq!(name, "my-profile");
        } else {
            panic!("Expected SaveProfile method");
        }
    }

    #[test]
    fn test_request_subscribe() {
        let request = Request {
            id: 7,
            method: Method::Subscribe { events: vec!["VolumeChanged".into(), "AppRouted".into()] },
        };

        let parsed = roundtrip_request(&request);
        if let Method::Subscribe { events } = parsed.method {
            assert_eq!(events.len(), 2);
            assert!(events.contains(&"VolumeChanged".to_string()));
            assert!(events.contains(&"AppRouted".to_string()));
        } else {
            panic!("Expected Subscribe method");
        }
    }

    #[test]
    fn test_request_shutdown() {
        let request = Request { id: 8, method: Method::Shutdown };

        let parsed = roundtrip_request(&request);
        assert!(matches!(parsed.method, Method::Shutdown));
    }

    #[test]
    fn test_response_success() {
        let response =
            Response { id: 1, result: Ok(serde_json::json!({"success": true, "volume": 0.5})) };
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""volume":0.5"#));

        let parsed = roundtrip_response(&response);
        assert_eq!(parsed.id, 1);
        assert!(parsed.result.is_ok());

        let value = parsed.result.unwrap();
        assert_eq!(value["success"], true);
    }

    #[test]
    fn test_response_error() {
        let response =
            Response { id: 2, result: Err(ErrorInfo::new(404, "Channel not found: unknown")) };
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""code":404"#));
        assert!(json.contains(r"Channel not found"));

        let parsed = roundtrip_response(&response);
        assert_eq!(parsed.id, 2);
        assert!(parsed.result.is_err());

        let error = parsed.result.unwrap_err();
        assert_eq!(error.code, 404);
        assert!(error.message.contains("Channel not found"));
    }

    #[test]
    fn test_error_info_new() {
        let error = ErrorInfo::new(500, "Internal server error");
        assert_eq!(error.code, 500);
        assert_eq!(error.message, "Internal server error");
    }

    #[test]
    fn test_all_simple_methods() {
        // Test that all simple (no params) methods serialize/deserialize
        let methods = [
            Method::GetState,
            Method::GetChannels,
            Method::GetApps,
            Method::GetProfiles,
            Method::GetDeviceStatus,
            Method::GetDiagnostics,
            Method::GetOutputDevices,
            Method::Shutdown,
            Method::Reconcile,
        ];

        for (i, method) in methods.into_iter().enumerate() {
            let request = Request { id: i as u64, method };
            let parsed = roundtrip_request(&request);
            assert_eq!(parsed.id, i as u64);
        }
    }

    #[test]
    fn test_mix_type_serialization() {
        // Test Stream mix type (serializes as lowercase "stream")
        let stream_request = Request {
            id: 1,
            method: Method::SetMasterVolume { mix: MixType::Stream, volume: 1.0 },
        };
        let json = serde_json::to_string(&stream_request).unwrap();
        assert!(json.contains(r#""mix":"stream""#));

        // Test Monitor mix type (serializes as lowercase "monitor")
        let monitor_request = Request {
            id: 2,
            method: Method::SetMasterMute { mix: MixType::Monitor, muted: false },
        };
        let json = serde_json::to_string(&monitor_request).unwrap();
        assert!(json.contains(r#""mix":"monitor""#));
    }

    #[test]
    fn test_request_from_json_string() {
        let json = r#"{"id":42,"method":{"type":"GetChannel","params":{"name":"browser"}}}"#;
        let request: Request = serde_json::from_str(json).expect("Failed to parse request JSON");

        assert_eq!(request.id, 42);
        if let Method::GetChannel { name } = request.method {
            assert_eq!(name, "browser");
        } else {
            panic!("Expected GetChannel method");
        }
    }

    #[test]
    fn test_response_from_json_string() {
        // Test successful response
        let ok_json = r#"{"id":1,"result":{"Ok":{"channels":[{"name":"music"}]}}}"#;
        let response: Response =
            serde_json::from_str(ok_json).expect("Failed to parse OK response");
        assert_eq!(response.id, 1);
        assert!(response.result.is_ok());

        // Test error response
        let err_json = r#"{"id":2,"result":{"Err":{"code":404,"message":"Not found"}}}"#;
        let response: Response =
            serde_json::from_str(err_json).expect("Failed to parse error response");
        assert_eq!(response.id, 2);
        assert!(response.result.is_err());
    }
}
