use anyhow::Result;
use serde_json::Value;
use rppal::gpio::OutputPin;
use crate::hal::devices::device::Device;

pub struct Led {
    pin: OutputPin,
    is_on: bool,
    pwm_frequency: f64,
    pwm_duty_cycle: f64,
}

impl Led {
    pub fn new(pin: OutputPin) -> Led {
        Led { pin, is_on: false, pwm_frequency: 0.0, pwm_duty_cycle: 0.0 }
    }

    pub fn toggle(&mut self) -> Result<bool> {
        if self.is_on {
            self.set_pwm(0.0)?;
            Ok(false)
        } else {
            self.set_pwm(1.0)?;
            Ok(true)
        }
    }

    pub fn turn_on(&mut self) -> Result<()> {
        if !self.is_on {
            self.set_pwm(1.0)?;
        }
        Ok(())
    }

    pub fn turn_off(&mut self) -> Result<()> {
        if self.is_on {
            self.set_pwm(0.0)?;
        }
        Ok(())
    }

    pub fn set_pwm_frequency(&mut self, frequency: f64, duty_cycle: f64) -> Result<()>{
        if duty_cycle == 0.0 || frequency == 0.0 {
            self.is_on = false;
        } else {
            self.is_on = true;
        }
        self.pwm_duty_cycle = duty_cycle;
        self.pwm_frequency = frequency;
        self.pin.set_pwm_frequency(frequency, duty_cycle)?;
        Ok(())
    }

    pub fn set_pwm(&mut self, duty_cycle: f64) -> Result<()>{
        if duty_cycle == 0.0 {
            self.is_on = false;
        } else {
            self.is_on = true;
        }
        self.pwm_duty_cycle = duty_cycle;
        self.pwm_frequency = 490.0;
        self.pin.set_pwm_frequency(490.0, duty_cycle)?;
        Ok(())
    }

    pub fn stop_pwm(&mut self)  -> Result<()>{
        self.is_on = false;
        self.pwm_duty_cycle = 0.0;
        self.pwm_frequency = 0.0;
        self.pin.set_pwm_frequency(0.0, 0.0)?;
        Ok(())
    }
}

impl Device for Led {
    fn handle(&mut self, cmd: &str, params: &Value) -> Result<Value, String> {
        match cmd {
            "pwm" => {
                println!("Got duty_cycle = {}", &params.get("duty_cycle").ok_or("duty_cycle missing")?);
                let duty_cycle = params.get("duty_cycle").and_then(|v| v.as_f64()).ok_or("missing or invalid 'duty_cycle'")?;
                let enable = params.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);

                if duty_cycle < 0.0 || duty_cycle > 1.0 {
                    return Err("invalid parameters: frequency must be >= 0, duty_cycle between 0.0 and 1.0".into());
                }

                if enable {
                    self.set_pwm(duty_cycle).map_err(|e| format!("failed to set pwm: {}", e))?;
                    Ok(serde_json::json!({"status":"ok","duty_cycle":duty_cycle}))
                } else {
                    self.stop_pwm().map_err(|e| format!("failed to stop pwm: {}", e))?;
                    Ok(serde_json::json!({"status":"stopped"}))
                }
            }
            "pwm_frequency" => {
                let frequency = params.get("frequency").and_then(|v| v.as_f64()).ok_or("missing or invalid 'frequency'")?;
                let duty_cycle = params.get("duty_cycle").and_then(|v| v.as_f64()).ok_or("missing or invalid 'duty_cycle'")?;
                let enable = params.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);

                if duty_cycle < 0.0 || duty_cycle > 1.0 || frequency < 0.0 {
                    return Err("invalid parameters: frequency must be >= 0, duty_cycle between 0.0 and 1.0".into());
                }

                if enable {
                    self.set_pwm_frequency(frequency, duty_cycle).map_err(|e| format!("failed to set pwm: {}", e))?;
                    Ok(serde_json::json!({"status":"ok","frequency":frequency,"duty_cycle":duty_cycle}))
                } else {
                    self.stop_pwm().map_err(|e| format!("failed to stop pwm: {}", e))?;
                    Ok(serde_json::json!({"status":"stopped"}))
                }
            }
            "turn_on" => {
                self.turn_on().map_err(|e| format!("failed to turn on led: {}", e))?;
                Ok(serde_json::json!({"status":"Ok", "is_on": true}))
            }
            "turn_off" => {
                self.turn_off().map_err(|e| format!("failed to turn off led: {}", e))?;
                Ok(serde_json::json!({"status":"Ok", "is_on": false}))
            }
            "toggle" => {
                let is_on = self.toggle().map_err(|e| format!("failed to toggle led: {}", e))?;
                Ok(serde_json::json!({"status":"Ok", "is_on": is_on}))
            }
            "get_state" => {
                let frequency = &self.pwm_frequency;
                let duty_cycle = &self.pwm_duty_cycle;
                Ok(serde_json::json!({"status":"Ok", "frequency":frequency, "duty_cycle":duty_cycle}))
            }
            other => Err(format!("unknown command for Led: {}", other)),
        }
    }
}
