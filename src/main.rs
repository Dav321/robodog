#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod model;
mod net;
mod peripheral;

use crate::net::app::{app_task, AppProps, WEB_TASK_POOL_SIZE};
use crate::net::network::{net_task, Network};
use crate::peripheral::cyw43::{cyw43_task, Cyw43};
use crate::peripheral::servo::{servo_task, Servo, ServoConfig};
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pwm::{Config, Pwm};
use embassy_rp::{bind_interrupts, init};
use embassy_time::{Duration, Timer};
use picoserve::{make_static, AppRouter, AppWithStateBuilder};
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

    let hz = 50;
    let div = 48;
    let top = (embassy_rp::clocks::clk_sys_freq() / hz) / div;
    let mut pwm_config = Config::default();
    pwm_config.divider = (div as u8).into();
    pwm_config.top = top as u16;

    let (pwm_0, pwm_1) = Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, pwm_config.clone()).split();
    let (pwm_2, _) = Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_2, p.PIN_3, pwm_config.clone()).split();

    let servo_config = ServoConfig::new(1.0/20.0, 2.0/20.0, 180);
    let servo_0 = Servo::new(pwm_0.unwrap(), servo_config.clone());
    let servo_1 = Servo::new(pwm_1.unwrap(), servo_config.clone());
    let servo_2 = Servo::new(pwm_2.unwrap(), servo_config.clone());

    spawner.must_spawn(servo_task(servo_0, servo_1, servo_2));

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
