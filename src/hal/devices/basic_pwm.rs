
use anyhow::Result;
use serde_json::Value;
use rppal::gpio::OutputPin;
use crate::hal::devices::device::Device;

pub struct BasicPWM {
    pin: OutputPin,
    is_on: bool,
    pwm_frequency: f64,
    pwm_duty_cycle: f64,
}

impl BasicPWM {
    pub fn new(pin: OutputPin) -> BasicPWM {
        BasicPWM { pin, is_on: false, pwm_frequency: 0.0, pwm_duty_cycle: 0.0 }
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

impl Device for BasicPWM {
    fn handle(&mut self, cmd: &str, params: &Value) -> Result<Value, String> {
        match cmd {
            "pwm" => {
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
            "stop" => {
                self.stop_pwm().map_err(|e| format!("failed to stop pwm: {e}"))?;
                Ok(serde_json::json!({"status":"Ok", "stopped": true}))
            }
            "get_state" => {
                let frequency = &self.pwm_frequency;
                let duty_cycle = &self.pwm_duty_cycle;
                Ok(serde_json::json!({"status":"Ok", "frequency":frequency, "duty_cycle":duty_cycle}))
            }
            other => Err(format!("unknown command for BasicPWM: {}", other)),
        }
    }
}
