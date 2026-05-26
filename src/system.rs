use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::format;
use std::time::{Duration, SystemTime};

use crate::hal::devices::device::Device;

/// System device that always exists.
///
/// Can be configured via devices.json by including a "system" entry:
/// ```json
/// [
///   {
///     "device": "system",
///     "device_type": "system",
///     "settings": {
///       "heartbeat_interval": 5,
///       "log_level": "info",
///       "custom_message": "Hello from rov-pi"
///     }
///   }
/// ]
/// ```
pub struct System {
    created_at: Duration,
    heartbeat_interval: u64,
    log_level: String,
    custom_message: String,
    devices: HashMap<String, String>,
}

impl System {
    pub fn new(devices: HashMap<String, String>) -> Self {
        System {
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO),
            heartbeat_interval: 5,
            log_level: "info".to_string(),
            custom_message: "rov-pi system".to_string(),
            devices,
        }
    }

    pub fn with_settings(devices: HashMap<String, String>, settings: &Value) -> Self {
        let settings = settings.as_object();

        System {
            created_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or(Duration::ZERO),
            heartbeat_interval: settings.and_then(|s| s.get("heartbeat_interval"))
                .and_then(|v| v.as_u64())
                .unwrap_or(5),
            log_level: settings.and_then(|s| s.get("log_level"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or("info".to_string()),
            custom_message: settings.and_then(|s| s.get("custom_message"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or("rov-pi system".to_string()),
            devices,
        }
    }
}

impl Device for System {
    fn handle(&mut self, cmd: &str, _params: &Value) -> Result<Value, String> {
        match cmd {
            "ping" => {
                Ok(serde_json::json!({
                    "status": "ok",
                    "response": "pong"
                }))
            }
            "get_info" => {
                Ok(serde_json::json!({
                    "status": "ok",
                    "created_at": self.created_at.as_secs(),
                    "uptime_seconds": SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0) - self.created_at.as_secs(),
                    "heartbeat_interval": self.heartbeat_interval,
                }))
            }
            "get_state" => {
                Ok(serde_json::json!({
                    "status": "ok",
                    "settings": serde_json::json!({
                        "heartbeat_interval": self.heartbeat_interval,
                        "log_level": self.log_level,
                        "custom_message": self.custom_message,
                    })
                }))
            }
            "heartbeat" => {
                Ok(serde_json::json!({
                    "status": "ok",
                    "message": self.custom_message
                }))
            }
            "list_devices" => {
                let mut devices_map: Vec<Value> = Vec::new();
                for (id, device_type) in &self.devices {
                    devices_map.push(
                        serde_json::json!({ "id": id, "device_type": device_type}),
                    );
                }
                Ok(serde_json::json!(devices_map))
            }
            _ => Err(format!("unknown system command: {}", cmd)),
        }
    }
}
