#![no_std]
#![no_main]

mod peripheral;

use core::time::Duration;
use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::{bind_interrupts, init};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::pwm::{PioPwm, PioPwmProgram};
use embassy_time::Timer;
use crate::peripheral::led::{let_task, LED};
use crate::peripheral::servo::Servo;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = init(Default::default());
    let Pio { mut common, sm0, sm1, .. } = Pio::new(p.PIO0, Irqs);


    let delay = embassy_time::Duration::from_secs(1);
    let led = LED::new(p.PIN_25);
    spawner.must_spawn(let_task(led, delay));

    let period = Duration::from_millis(20);
    let min_pw = Duration::from_micros(900);
    let max_pw = Duration::from_micros(2100);
    let pwm_program = PioPwmProgram::new(&mut common);

    let upper_servo_pwm = PioPwm::new(&mut common, sm0, p.PIN_2, &pwm_program);
    let mut upper_servo = Servo::new(upper_servo_pwm, period, min_pw, max_pw, 180);

    let lower_servo_pwm = PioPwm::new(&mut common, sm1, p.PIN_3, &pwm_program);
    let mut lower_servo = Servo::new(lower_servo_pwm, period, min_pw, max_pw, 180);

    upper_servo.start();
    lower_servo.start();

    loop {
        upper_servo.rotate(0);
        lower_servo.rotate(180);
        info!("0");
        Timer::after(delay).await;

        upper_servo.rotate(180);
        lower_servo.rotate(0);
        info!("1");
        Timer::after(delay).await;
    }
}
