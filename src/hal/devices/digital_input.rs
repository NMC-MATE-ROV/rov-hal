use anyhow::Result;
use serde_json::Value;

use rppal::gpio::{InputPin};

use crate::hal::devices::device::Device;

pub struct DigitalInput {
    pin: InputPin,
    /// defaults to false (High is 1)
    inverted: bool
}

impl DigitalInput {
    pub fn new(pin: InputPin, inverted: Option<bool>) -> DigitalInput {
        DigitalInput { pin, inverted: inverted.unwrap_or(false)}
    }
}

impl Device for DigitalInput {
    fn handle(&mut self, cmd: &str, _params: &Value) -> Result<Value, String> {
        match cmd {
            "get" => {
                if !self.inverted {
                    let value = self.pin.is_high();
                    Ok(serde_json::json!({"value": value}))
                } else {
                    let value = self.pin.is_low();
                    Ok(serde_json::json!({"value": value}))
                }
            }
            other => Err(format!("unknown command for DigitalInput: {}", other)),
        }
    }
}
