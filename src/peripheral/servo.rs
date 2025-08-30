use crate::model::ik::{IkSolver, Joint};
use core::time::Duration;
use defmt::info;
use embassy_futures::select::{Either, select};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::Instance;
use embassy_rp::pio_programs::pwm::PioPwm;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub static SERVO_SIGNAL: Signal<CriticalSectionRawMutex, (u16, u16)> = Signal::new();
pub static SERVO_CALIBRATION_SIGNAL: Signal<CriticalSectionRawMutex, u16> = Signal::new();
#[embassy_executor::task]
pub async fn servo_task(
    mut upper_servo: Servo<'static, PIO0, 1>,
    mut lower_servo: Servo<'static, PIO0, 2>,
) -> ! {
    let solver = IkSolver::new(Joint::new(100f32), Joint::new(100f32));

    loop {
        match select(SERVO_SIGNAL.wait(), SERVO_CALIBRATION_SIGNAL.wait()).await {
            Either::First((x, y)) => {
                let (a1, a2) = solver.solve(x as f32, y as f32);
                info!("[Servo] pos: ({}|{}) angle: {} - {}", x, y, a1, a2);
                upper_servo.rotate(a1);
                lower_servo.rotate(a2);
            }
            Either::Second(pwm) => {
                info!("[Servo] cal: {}", pwm);
                upper_servo.write(Duration::from_micros(pwm as u64));
                lower_servo.write(Duration::from_micros(pwm as u64));
            }
        }
    }
}

pub struct Servo<'d, T: Instance, const SM: usize> {
    pwm: PioPwm<'d, T, SM>,

    period: Duration,
    min_pw: Duration,
    max_pw: Duration,

    max_rotation: u64,
}

impl<'d, T: Instance, const SM: usize> Servo<'d, T, SM> {
    pub fn new(
        pwm: PioPwm<'d, T, SM>,
        period: Duration,
        min_pw: Duration,
        max_pw: Duration,
        max_rotation: u64,
    ) -> Self {
        Self {
            pwm,
            period,
            min_pw,
            max_pw,
            max_rotation,
        }
    }

    pub fn ky66(pwm: PioPwm<'d, T, SM>, min_pw: u64, max_pw: u64, max_rotation: u64) -> Self {
        let period = Duration::from_millis(20);
        let min_pw = Duration::from_micros(min_pw);
        let max_pw = Duration::from_micros(max_pw);

        Self::new(pwm, period, min_pw, max_pw, max_rotation)
    }

    pub fn start(&mut self) {
        self.pwm.set_period(self.period);
        self.pwm.start();
    }

    pub fn rotate(&mut self, degree: u8) {
        let pw_ns_diff = self.max_pw.as_nanos() as u64 - self.min_pw.as_nanos() as u64;
        let deg_per_ns = pw_ns_diff / self.max_rotation;

        let mut duration =
            Duration::from_nanos(degree as u64 * deg_per_ns + self.min_pw.as_nanos() as u64);

        if self.max_pw < duration {
            duration = self.max_pw;
        }

        self.pwm.write(duration);
    }

    pub fn write(&mut self, duration: Duration) {
        self.pwm.write(duration);
    }

    #[allow(unused)]
    pub fn stop(&mut self) {
        self.pwm.stop();
    }
}
