use core::time::Duration;
use embassy_rp::pio::Instance;
use embassy_rp::pio_programs::pwm::PioPwm;

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

    pub fn stop(&mut self) {
        self.pwm.stop();
    }
}
