use anyhow::Result;
use serde_json::Value;

use rppal::gpio::OutputPin;

use crate::hal::devices::device::Device;

pub struct Servo {
    pin: OutputPin,
    pwm_frequency: f64,
    pwm_duty_cycle: f64,
}

impl Servo {
    pub fn new(pin: OutputPin) -> Servo {
        Servo { pin, pwm_frequency: 0.0, pwm_duty_cycle: 0.0 }
    }

    pub fn set_pwm(&mut self, frequency: f64, duty_cycle: f64) -> Result<()>{
        self.pwm_duty_cycle = duty_cycle;
        self.pwm_frequency = frequency;
        self.pin.set_pwm_frequency(frequency, duty_cycle)?;
        Ok(())
    }

    pub fn stop_pwm(&mut self)  -> Result<()>{
        self.pwm_duty_cycle = 0.0;
        self.pwm_frequency = 0.0;
        self.pin.set_pwm_frequency(0.0, 0.0)?;
        Ok(())
    }
}

impl Device for Servo {
    fn handle(&mut self, cmd: &str, params: &Value) -> Result<Value, String> {
        match cmd {
            "pwm" => {
                let frequency = params.get("frequency").and_then(|v| v.as_f64()).ok_or("missing or invalid 'frequency'")?;
                let duty_cycle = params.get("duty_cycle").and_then(|v| v.as_f64()).ok_or("missing or invalid 'duty_cycle'")?;

                if duty_cycle < 0.0 || duty_cycle > 1.0 || frequency < 0.0 {
                    return Err("invalid parameters: frequency must be >= 0, duty_cycle between 0.0 and 1.0".into());
                }

                self.set_pwm(frequency, duty_cycle).map_err(|e| format!("failed to set pwm: {}", e))?;
                Ok(serde_json::json!({"status":"ok","frequency":frequency,"duty_cycle":duty_cycle}))
            }
            other => Err(format!("unknown command for Servo: {}", other)),
        }
    }
}
