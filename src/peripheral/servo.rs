use crate::model::ik::{IkSolver, Joint};
use core::f32::consts::PI;
use defmt::{Format, Formatter, debug, error, info, write};
use embassy_futures::select::{Either, select};
use embassy_rp::pwm::PwmOutput;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embedded_hal::pwm::SetDutyCycle;
use libm::cosf;

pub enum ServoTask {
    CALIBRATION(u8, f32),
    MOVE(f32, f32, f32),
    HOME,
}

impl Format for ServoTask {
    fn format(&self, fmt: Formatter) {
        match self {
            ServoTask::CALIBRATION(servo, pos) => {
                write!(fmt, "CALIBRATION({} -> {})", servo, pos)
            }
            ServoTask::MOVE(x, y, z) => {
                write!(fmt, "MOVE({})", (x, y, z))
            }
            ServoTask::HOME => {
                write!(fmt, "HOME")
            }
        }
    }
}

pub static SERVO_SIGNAL: Signal<CriticalSectionRawMutex, ServoTask> = Signal::new();
#[embassy_executor::task]
pub async fn servo_task(mut servos: [Servo<'static>; 12]) -> ! {
    let delay = Duration::from_millis(5);
    let solver = IkSolver::new(Joint::new(0.0), Joint::new(100.0), Joint::new(100.0));

    loop {
        match select(SERVO_SIGNAL.wait(), Timer::after(delay)).await {
            Either::First(task) => {
                info!("Task: {}", task);
                match task {
                    ServoTask::CALIBRATION(servo, pos) => {
                        if servo > 12 {
                            error!("Servo out of range: {}", servo);
                            continue;
                        }
                        servos[servo as usize].write(pos);
                    }
                    ServoTask::MOVE(x, y, z) => {
                        if let Some((a1, a2, a3)) = solver.solve(x, y, z) {
                            debug!("Servo signal: angles={}", (a1, a2, a3));
                            let mut i = 0u8;
                            for s in &mut servos {
                                i += 1;
                                match i {
                                    1 => s.rotate(a1),
                                    2 => s.rotate(a2),
                                    3 => {
                                        s.rotate(a3);
                                        i = 0;
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        } else {
                            error!("Not Reachable!");
                            SERVO_SIGNAL.signal(ServoTask::HOME);
                        };
                    }
                    ServoTask::HOME => {
                        for s in &mut servos {
                            s.home()
                        }
                    }
                }
            }
            Either::Second(()) => {
                for s in &mut servos {
                    s.tick()
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct ServoConfig {
    min_angle: f32,
    max_angle: f32,
    home: f32,

    /// clamp output
    calibrated: bool,

    max_rotation: u16,
    /// invert angle
    inverted: bool,
    /// offset added to the max_rotation after inversion
    offset: u16,
}

impl ServoConfig {
    pub fn new(
        min: f32,
        home: f32,
        max: f32,
        max_rotation: u16,
        offset: u16,
        inverted: bool,
        calibrated: bool,
    ) -> Self {
        ServoConfig {
            min_angle: min,
            max_angle: max,
            max_rotation,
            offset,
            inverted,
            calibrated,
            home,
        }
    }
}

pub struct Servo<'d> {
    pwm: PwmOutput<'d>,
    config: ServoConfig,

    step: f32,
    prev: u16,
    duty: u16,
    target: u16,
}

impl<'d> Servo<'d> {
    pub fn new(mut pwm: PwmOutput<'d>, config: ServoConfig) -> Self {
        let home = (config.home * pwm.max_duty_cycle() as f32) as u16;
        pwm.set_duty_cycle(home).expect("set_duty_cycle failed");
        Self {
            pwm,
            config,
            step: 1.0,
            prev: home,
            duty: home,
            target: home,
        }
    }

    pub fn home(&mut self) {
        self.write(self.config.home);
    }

    pub fn rotate(&mut self, degree: f32) {
        if !degree.is_finite() {
            debug!("NaN Requested! Homing...");
            SERVO_SIGNAL.signal(ServoTask::HOME)
        }
        let mut degree = degree;
        if self.config.inverted {
            degree = self.config.max_rotation as f32 - degree;
        }
        if self.config.offset != 0 {
            degree += self.config.offset as f32;
        }
        if degree.is_sign_negative() {
            error!("Degree is negative, assuming 0.0: {}", degree);
            degree = 0.0;
        }

        let max_deg = self.config.max_rotation as f32;
        let delta = self.config.max_angle - self.config.min_angle;
        let degree_percent = degree / max_deg;
        let percentage = self.config.min_angle + (degree_percent * delta);
        self.write(percentage);
    }

    pub fn write(&mut self, percentage: f32) {
        let mut percentage = percentage;
        if percentage > 1.0 || percentage < 0.0 {
            error!("Percentage must be between 0 and 1, is: {}", percentage);
            percentage = percentage.clamp(0.0, 1.0);
        }
        if self.config.calibrated
            && (percentage > self.config.max_angle || percentage < self.config.min_angle)
        {
            let clamped = percentage.clamp(self.config.min_angle, self.config.max_angle);
            error!(
                "Percentage {} out of range, Clamped: {}",
                percentage, clamped
            );
            percentage = clamped;
        }
        let max = self.pwm.max_duty_cycle();
        let val = (percentage * max as f32) as u16;
        self.target = val;
        self.prev = self.duty;
        self.step = 0.0;
        debug!("Set next duty cycle to={}", val);
        self.tick();
    }
    pub fn tick(&mut self) {
        if self.step < 1.0 {
            let inc = 1.0 / 150.0;
            self.step += inc;
        } else {
            return;
        }

        self.duty = self.ease_sine(self.step, self.prev as f32, self.target as f32) as u16;

        self.pwm.set_duty_cycle(self.duty).unwrap();
    }

    fn ease_sine(&mut self, x: f32, start: f32, end: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        let sine = -(cosf(PI * x) - 1.0) / 2.0;
        let diff = end - start;
        start + (diff * sine)
    }
}
