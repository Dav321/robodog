use core::time::Duration;
use defmt::info;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::Instance;
use embassy_rp::pio_programs::pwm::PioPwm;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;

pub static UPPER_SERVO_SIGNAL: Signal<CriticalSectionRawMutex, u64> = Signal::new();
#[embassy_executor::task]
pub async fn upper_servo_task(mut servo: Servo<'static, PIO0, 1>) -> ! {
    loop {
        let angle = UPPER_SERVO_SIGNAL.wait().await;
        info!("Upper servo: {}", angle);
        servo.rotate(angle);
    }
}
pub static LOWER_SERVO_SIGNAL: Signal<CriticalSectionRawMutex, u64> = Signal::new();
#[embassy_executor::task]
pub async fn lower_servo_task(mut servo: Servo<'static, PIO0, 2>) -> ! {
    loop {
        let angle = LOWER_SERVO_SIGNAL.wait().await;
        info!("Lower servo: {}", angle);
        servo.rotate(angle);
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

    pub fn ky66(pwm: PioPwm<'d, T, SM>) -> Self {
        let period = Duration::from_millis(20);
        let min_pw = Duration::from_micros(900);
        let max_pw = Duration::from_micros(2100);

        Self::new(pwm, period, min_pw, max_pw, 180)
    }

    pub fn start(&mut self) {
        self.pwm.set_period(self.period);
        self.pwm.start();
    }

    pub fn rotate(&mut self, degree: u64) {
        let pw_ns_diff = self.max_pw.as_nanos() as u64 - self.min_pw.as_nanos() as u64;
        let deg_per_ns = pw_ns_diff / self.max_rotation;

        let mut duration =
            Duration::from_nanos(degree * deg_per_ns + self.min_pw.as_nanos() as u64);

        if self.max_pw < duration {
            duration = self.max_pw;
        }

        self.pwm.write(duration);
    }

    #[allow(unused)]
    pub fn stop(&mut self) {
        self.pwm.stop();
    }
}
