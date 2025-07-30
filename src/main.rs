#![no_std]
#![no_main]

mod peripheral;
mod net;

use core::time::Duration;
use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, init};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::pwm::{PioPwm, PioPwmProgram};
use embassy_time::Timer;
use crate::net::network::{net_task, Network};
use crate::peripheral::cyw43::{cyw43_task, Cyw43};
use crate::peripheral::servo::Servo;

const WIFI_NETWORK: &str = include_str!("../secrets/wifi_ssid.txt");
const WIFI_PASSWORD: &str = include_str!("../secrets/wifi_pw.txt");

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = init(Default::default());
    let Pio { mut common, irq0, sm0, sm1, sm3, .. } = Pio::new(p.PIO0, Irqs);

    let (mut cyw43, net_device, cyw43_runner) = Cyw43::new(&mut common, sm0, irq0, p.DMA_CH0, p.PIN_23, p.PIN_25, p.PIN_24, p.PIN_29).await;
    spawner.must_spawn(cyw43_task(cyw43_runner));

    cyw43.init().await;
    let (mut net, net_runner) = Network::new(net_device);
    spawner.must_spawn(net_task(net_runner));

    cyw43.join_wifi(WIFI_NETWORK, WIFI_PASSWORD).await;
    net.up().await;

    let period = Duration::from_millis(20);
    let min_pw = Duration::from_micros(900);
    let max_pw = Duration::from_micros(2100);
    let pwm_program = PioPwmProgram::new(&mut common);

    let upper_servo_pwm = PioPwm::new(&mut common, sm1, p.PIN_2, &pwm_program);
    let mut upper_servo = Servo::new(upper_servo_pwm, period, min_pw, max_pw, 180);

    let lower_servo_pwm = PioPwm::new(&mut common, sm3, p.PIN_3, &pwm_program);
    let mut lower_servo = Servo::new(lower_servo_pwm, period, min_pw, max_pw, 180);

    upper_servo.start();
    lower_servo.start();

    let delay = embassy_time::Duration::from_secs(1);
    loop {
        cyw43.set_led(false).await;
        info!("0");
        Timer::after(delay).await;

        cyw43.set_led(true).await;
        info!("1");
        Timer::after(delay).await;
    }
}
