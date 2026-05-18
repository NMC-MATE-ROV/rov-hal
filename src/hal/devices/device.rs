use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use anyhow::Result;

use crate::hal::devices::{blue_esc::BlueEsc, led::Led, servo::Servo};
use rppal::gpio::Gpio;

#[derive(Deserialize)]
pub struct RawDevice {
    pub id: String,
    pub device_type: String,
    pub pin: u8,
}

/// Device trait used by the runtime to dispatch commands to devices.
pub trait Device: Send + Sync {
    /// Handle a command with optional params. Returns a JSON-like Value on success or an error string.
    fn handle(&mut self, cmd: &str, params: &Value) -> Result<Value, String>;
}

pub type SharedDevice = Arc<Mutex<Box<dyn Device + Send + Sync>>>;
pub type DevicesMap = HashMap<String, SharedDevice>;
pub type DevicesRegistry = Arc<Mutex<DevicesMap>>;

/// Create devices from a RawDevice list and register their GPIO pins.
/// Returns a map keyed by device id containing shared device handles.
pub fn create_gpio_devices(devices: &Vec<RawDevice>, gpio: &Gpio) -> Result<DevicesMap> {
    let mut map: DevicesMap = HashMap::new();

    for d in devices.iter() {
        // Create the output pin (assume output for now)
        let pin = gpio.get(d.pin)?.into_output();

        match d.device_type.as_str() {
            "led" => {
                let led = Led::new(pin);
                let boxed: Box<dyn Device + Send + Sync> = Box::new(led);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            "servo" => {
                let servo = Servo::new(pin);
                let boxed: Box<dyn Device + Send + Sync> = Box::new(servo);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            "blue_esc" => {
                let blue_esc = BlueEsc::new(pin);
                let boxed: Box<dyn Device + Send + Sync> = Box::new(blue_esc);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            other => {
                // unknown device type; skip or log
                return Err(anyhow::anyhow!(format!("unknown device type: {}", other)));
            }
        }
    }

    Ok(map)
}
