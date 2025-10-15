use defmt::{debug, error};
use embassy_rp::pwm::PwmOutput;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embedded_hal::pwm::SetDutyCycle;


pub static SERVO_SIGNAL: Signal<CriticalSectionRawMutex, (u16, u16, u16)> = Signal::new();
pub static SERVO_CALIBRATION_SIGNAL: Signal<CriticalSectionRawMutex, f32> = Signal::new();
#[embassy_executor::task]
pub async fn servo_task(mut servo_0: Servo<'static>, mut servo_1: Servo<'static>, mut servo_2: Servo<'static>) -> ! {
    loop {
        let pos = SERVO_CALIBRATION_SIGNAL.wait().await;
        debug!("Servo calibration signal: pos={}", pos);
        servo_0.write(pos);
        servo_1.write(pos);
        servo_2.write(pos);
    }
}

#[derive(Copy, Clone)]
pub struct ServoConfig {
    min: f32,
    max: f32,

    max_rotation: u16,
}

impl ServoConfig {
    pub fn new(min: f32, max: f32, max_rotation: u16) -> Self {
        ServoConfig { min, max, max_rotation }
    }
}

pub struct Servo<'d> {
    pwm: PwmOutput<'d>,
    config: ServoConfig,
}

impl<'d> Servo<'d> {
    pub fn new(
        pwm: PwmOutput<'d>,
        config: ServoConfig
    ) -> Self {
        Self {
            pwm,
            config,
        }
    }

    pub fn rotate(&mut self, degree: f32) {
        let max_deg = self.config.max_rotation as f32;
        let delta = self.config.max - self.config.min;
        let degree_percent = degree / max_deg;
        let percentage = self.config.min + (degree_percent * delta);
        self.write(percentage);
    }

    pub fn write(&mut self, percentage: f32) {
        if percentage > 1.0 || percentage < 0.0 {
            error!("Percentage must be between 0 and 1, is: {}", percentage);
            return;
        }
        let max = self.pwm.max_duty_cycle();
        let val = (percentage * max as f32) as u16;
        self.pwm.set_duty_cycle(val).unwrap();
        debug!("Set duty cycle to={}", val);
    }
}
