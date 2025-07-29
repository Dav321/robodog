#![no_std]
#![no_main]

use defmt::info;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::init;
use embassy_time::{Duration, Timer};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);
    let delay = Duration::from_secs(1);

    loop {
        led.set_high();
        info!("1");
        Timer::after(delay).await;

        led.set_low();
        info!("0");
        Timer::after(delay).await;
    }
}
