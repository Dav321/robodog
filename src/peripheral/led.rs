use cortex_m::prelude::_embedded_hal_digital_ToggleableOutputPin;
use embassy_rp::gpio::{Level, Output, Pin};
use embassy_rp::Peri;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async  fn let_task(mut led: LED<'static>, delay: Duration) {
    loop {
        led.toggle();
        Timer::after(delay).await;
    }
}

pub struct LED<'d> {
    led: Output<'d>
}

impl<'d> LED<'d> {
    pub fn new(pin: Peri<'d, impl Pin>) -> Self {
        let output = Output::new(pin, Level::Low);
        Self { led: output }
    }
    
    pub fn toggle(&mut self) {
        self.led.toggle();
    }
}