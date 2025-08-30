#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod model;
mod net;
mod peripheral;

use crate::net::app::{AppProps, WEB_TASK_POOL_SIZE, app_task};
use crate::net::network::{Network, net_task};
use crate::peripheral::cyw43::{Cyw43, cyw43_task};
use crate::peripheral::servo::{Servo, servo_task};
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pio_programs::pwm::{PioPwm, PioPwmProgram};
use embassy_rp::{bind_interrupts, init};
use embassy_time::{Duration, Timer};
use picoserve::{AppRouter, AppWithStateBuilder, make_static};
#[allow(unused)]
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = init(Default::default());
    let Pio {
        mut common,
        irq0,
        sm0,
        sm1,
        sm2,
        ..
    } = Pio::new(p.PIO0, Irqs);

    let (mut cyw43, net_device, cyw43_runner) = Cyw43::new(
        &mut common,
        sm0,
        irq0,
        p.DMA_CH0,
        p.PIN_23,
        p.PIN_25,
        p.PIN_24,
        p.PIN_29,
    )
    .await;
    spawner.must_spawn(cyw43_task(cyw43_runner));

    cyw43.init().await;
    let (mut net, net_runner) = Network::new(net_device);
    spawner.must_spawn(net_task(net_runner));

    cyw43.create_ap("robodog_ap", "robodogg").await;
    net.up().await;

    let pwm_program = PioPwmProgram::new(&mut common);
    let mut upper_servo = Servo::ky66(
        PioPwm::new(&mut common, sm1, p.PIN_2, &pwm_program),
        1540,
        2708,
        90
    );
    let mut lower_servo = Servo::ky66(
        PioPwm::new(&mut common, sm2, p.PIN_3, &pwm_program),
        400,
        2389,
        180
    );

    upper_servo.start();
    lower_servo.start();
    spawner.must_spawn(servo_task(upper_servo, lower_servo));

    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());
    let config = make_static!(
        picoserve::Config<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            persistent_start_read_request: Some(Duration::from_secs(1)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(1)),
        })
        .keep_connection_alive()
    );

    for i in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(app_task(i, net.stack, app, config))
    }

    let delay = Duration::from_secs(1);
    loop {
        cyw43.set_led(false).await;
        Timer::after(delay).await;

        cyw43.set_led(true).await;
        Timer::after(delay).await;
    }
}
