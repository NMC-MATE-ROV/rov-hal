use std::time::Duration;

use anyhow::Result;
use serde_json::Value;
use rppal::gpio::OutputPin;
use crate::hal::devices::device::Device;

pub struct BlueEsc {
    pin: OutputPin,
    is_enabled: bool,
    duty_cycle: u64,
}

const DUTY_CYCLE_CENTER: u64 = 1500;
const DUTY_CYCLE_RANGE: u64 = 400;

impl BlueEsc {
    pub fn new(pin: OutputPin) -> BlueEsc {
        BlueEsc { pin, is_enabled: true, duty_cycle: 1500}
    }

    pub fn set_control(&mut self, duty_cycle: u64) -> Result<()>{
        self.duty_cycle = duty_cycle;
        self.pin.set_pwm(Duration::from_micros(460), Duration::from_micros(duty_cycle))?;
        Ok(())
    }

    pub fn stop(&mut self)  -> Result<()>{
        self.set_control(DUTY_CYCLE_CENTER)?;
        Ok(())
    }

    pub fn enable(&mut self) -> Result<()> {
        self.set_control(self.duty_cycle)?;
        self.is_enabled = true;
        Ok(())
    }

    pub fn disable(&mut self) -> Result<()> {
        self.stop()?;
        self.is_enabled = false;
        Ok(())
    }
}

impl Device for BlueEsc {
    fn handle(&mut self, cmd: &str, params: &Value) -> Result<Value, String> {
        match cmd {
            "set_control" => {
                println!("Got duty_cycle = {}", &params.get("duty_cycle").ok_or("duty_cycle missing")?);
                let duty_cycle_percent = params.get("duty_cycle").and_then(|v| v.as_f64()).ok_or("missing or invalid 'duty_cycle'")?;
                let enable = params.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);

                if duty_cycle_percent < -1.0 || duty_cycle_percent > 1.0 {
                    return Err("invalid parameters: duty_cycle must be between -1.0 and 1.0".into());
                }

                let duty_cycle = DUTY_CYCLE_CENTER + (DUTY_CYCLE_RANGE as f64 * duty_cycle_percent) as u64;

                if enable {
                    self.set_control(duty_cycle).map_err(|e| format!("failed to set pwm: {}", e))?;
                    Ok(serde_json::json!({"status":"ok","duty_cycle":duty_cycle_percent}))
                } else {
                    self.stop().map_err(|e| format!("failed to stop pwm: {}", e))?;
                    Ok(serde_json::json!({"status":"stopped"}))
                }
            }
            "enable" => {
                self.enable().map_err(|_| format!("failed to enable blue_esc"))?;
                Ok(serde_json::json!({"status":"Ok", "enabled":false}))
            }
            "disable" => {
                self.disable().map_err(|_| format!("failed to disable blue_esc"))?;
                Ok(serde_json::json!({"status":"Ok", "enabled":true}))
            }
            "get_state" => {
                let duty_cycle = &self.duty_cycle;
                let enabled = &self.is_enabled;
                Ok(serde_json::json!({"status":"Ok", "duty_cycle":duty_cycle, "enabled":enabled}))
            }
            other => Err(format!("unknown command for BlueEsc: {}", other)),
        }
    }
}
