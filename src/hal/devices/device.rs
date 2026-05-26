use anyhow::{Ok, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::hal::devices::{basic_pwm::BasicPWM, blue_esc::BlueEsc, digital_input::DigitalInput, led::Led, servo::Servo};
use rppal::gpio::Gpio;

#[derive(Deserialize)]
pub struct RawDevice {
    pub id: String,
    pub device_type: String,
    pub pin: u8,
    pub pullup: Option<bool>,
    pub input_inverted: Option<bool>,
    /// Optional JSON settings for device-specific configuration
    /// Example for system device:
    /// ```json
    /// {
    ///   "device": "system",
    ///   "device_type": "system",
    ///   "pin": 0,
    ///   "params": {
    ///     "heartbeat_interval": 5,
    ///     "log_level": "info"
    ///   }
    /// }
    /// ```
    pub params: Option<Value>,
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
        let pin = gpio.get(d.pin)?;

        match d.device_type.as_str() {
            "led" => {
                let led = Led::new(pin.into_output());
                let boxed: Box<dyn Device + Send + Sync> = Box::new(led);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            "servo" => {
                let servo = Servo::new(pin.into_output());
                let boxed: Box<dyn Device + Send + Sync> = Box::new(servo);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            "blue_esc" => {
                let blue_esc = BlueEsc::new(pin.into_output());
                let boxed: Box<dyn Device + Send + Sync> = Box::new(blue_esc);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            "basic_pwm" => {
                let basic_pwm = BasicPWM::new(pin.into_output());
                let boxed: Box<dyn Device + Send + Sync> = Box::new(basic_pwm);
                map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
            }
            "digital_input" => match d.pullup {
                Some(is_pullup) => {
                    if is_pullup {
                        let digital_input =
                            DigitalInput::new(pin.into_input_pullup(), d.input_inverted);
                        let boxed: Box<dyn Device + Send + Sync> = Box::new(digital_input);
                        map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
                    } else {
                        let digital_input =
                            DigitalInput::new(pin.into_input_pulldown(), d.input_inverted);
                        let boxed: Box<dyn Device + Send + Sync> = Box::new(digital_input);
                        map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
                    }
                }
                None => {
                    let digital_input = DigitalInput::new(pin.into_input(), d.input_inverted);
                    let boxed: Box<dyn Device + Send + Sync> = Box::new(digital_input);
                    map.insert(d.id.clone(), Arc::new(Mutex::new(boxed)));
                }
            },
            "system" => {
                // System device is created in main.rs with the devices map
            }
            other => {
                // unknown device type; skip or log
                return Err(anyhow::anyhow!(format!("unknown device type: {}", other)));
            }
        }
    }

    Ok(map)
}
