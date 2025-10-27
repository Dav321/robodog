#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod macros;
mod model;
mod net;
mod peripheral;

use crate::net::app::{AppProps, WEB_TASK_POOL_SIZE, app_task};
use crate::net::network::{Network, net_task};
use crate::peripheral::cyw43::{Cyw43, cyw43_task};
use crate::peripheral::servo::{Servo, ServoConfig, servo_task};
use embassy_executor::Spawner;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::pwm::{Config, Pwm};
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

    let (pwm_0, pwm_1) =
        Pwm::new_output_ab(p.PWM_SLICE0, p.PIN_0, p.PIN_1, pwm_config.clone()).split();
    let (pwm_2, _) = Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_2, p.PIN_3, pwm_config.clone()).split();
    let (pwm_4, pwm_5) =
        Pwm::new_output_ab(p.PWM_SLICE2, p.PIN_4, p.PIN_5, pwm_config.clone()).split();
    let (pwm_6, _) = Pwm::new_output_ab(p.PWM_SLICE3, p.PIN_6, p.PIN_7, pwm_config.clone()).split();
    let (pwm_8, pwm_9) =
        Pwm::new_output_ab(p.PWM_SLICE4, p.PIN_8, p.PIN_9, pwm_config.clone()).split();
    let (pwm_10, _) =
        Pwm::new_output_ab(p.PWM_SLICE5, p.PIN_10, p.PIN_11, pwm_config.clone()).split();
    let (pwm_12, pwm_13) =
        Pwm::new_output_ab(p.PWM_SLICE6, p.PIN_12, p.PIN_13, pwm_config.clone()).split();
    let (pwm_14, _) =
        Pwm::new_output_ab(p.PWM_SLICE7, p.PIN_14, p.PIN_15, pwm_config.clone()).split();

    #[allow(unused_variables)]
    let mg90s_config = ServoConfig::new(1.0 / 20.0, 1.5 / 20.0, 2.0 / 20.0, 180, 0, false, false);
    let servo_0 = Servo::new(
        pwm_0.unwrap(),
        ServoConfig::new(0.035, 0.06, 0.086, 90, 45, false, true),
    );
    let servo_1 = Servo::new(
        pwm_1.unwrap(),
        ServoConfig::new(0.03, 0.105, 0.12, 180 - 15, 90, false, true),
    );
    let servo_2 = Servo::new(
        pwm_2.unwrap(),
        ServoConfig::new(0.0315, 0.08, 0.12, 180 - 15, 0, true, true),
    );
    let servo_4 = Servo::new(pwm_4.unwrap(), mg90s_config);
    let servo_5 = Servo::new(pwm_5.unwrap(), mg90s_config);
    let servo_6 = Servo::new(pwm_6.unwrap(), mg90s_config);
    let servo_8 = Servo::new(pwm_8.unwrap(), mg90s_config);
    let servo_9 = Servo::new(pwm_9.unwrap(), mg90s_config);
    let servo_10 = Servo::new(pwm_10.unwrap(), mg90s_config);
    let servo_12 = Servo::new(pwm_12.unwrap(), mg90s_config);
    let servo_13 = Servo::new(pwm_13.unwrap(), mg90s_config);
    let servo_14 = Servo::new(pwm_14.unwrap(), mg90s_config);

    spawner.must_spawn(servo_task([
        servo_0, servo_1, servo_2, servo_4, servo_5, servo_6, servo_8, servo_9, servo_10, servo_12,
        servo_13, servo_14,
    ]));

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
