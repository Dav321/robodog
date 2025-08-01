use core::marker::PhantomData;
use cyw43::{Control, JoinOptions, NetDriver, Runner};
use cyw43_pio::{DEFAULT_CLOCK_DIVIDER, PioSpi};
use defmt::info;
use embassy_rp::Peri;
use embassy_rp::dma::Channel;
use embassy_rp::gpio::{Level, Output, Pin};
use embassy_rp::peripherals::{DMA_CH0, PIO0};
use embassy_rp::pio::{Common, Instance, Irq, PioPin, StateMachine};
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn cyw43_task(
    runner: Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

pub struct Cyw43<'d, T: Instance, const SM: usize, DMA: Channel> {
    control: Control<'d>,
    phantom: PhantomData<(T, DMA)>,
}

impl<'d, T: Instance, const SM: usize, DMA: Channel> Cyw43<'d, T, SM, DMA> {
    pub async fn new(
        common: &mut Common<'d, T>,
        sm: StateMachine<'d, T, SM>,
        irq: Irq<'d, T, 0>,
        dma: Peri<'d, DMA>,
        pwr: Peri<'d, impl Pin>,
        cs: Peri<'d, impl Pin>,
        dio: Peri<'d, impl PioPin>,
        clk: Peri<'d, impl PioPin>,
    ) -> (
        Cyw43<'d, T, SM, DMA>,
        NetDriver<'d>,
        Runner<'d, Output<'d>, PioSpi<'d, T, SM, DMA>>,
    ) {
        let fw = include_bytes!("../../firmware/43439A0.bin");

        let pwr = Output::new(pwr, Level::Low);
        let cs = Output::new(cs, Level::High);
        let spi = PioSpi::new(common, sm, DEFAULT_CLOCK_DIVIDER, irq, cs, dio, clk, dma);

        static NET_STATE: StaticCell<cyw43::State> = StaticCell::new();
        let state = NET_STATE.init(cyw43::State::new());
        let (net_device, control, runner) = cyw43::new(state, pwr, spi, fw).await;

        (
            Self {
                control,
                phantom: PhantomData,
            },
            net_device,
            runner,
        )
    }

    pub async fn init(&mut self) {
        let clm = include_bytes!("../../firmware/43439A0_clm.bin");
        self.control.init(clm).await;

        self.control
            .set_power_management(cyw43::PowerManagementMode::PowerSave)
            .await;
    }

    pub async fn join_wifi(&mut self, ssid: &str, password: &str) {
        loop {
            match self
                .control
                .join(ssid, JoinOptions::new(password.as_bytes()))
                .await
            {
                Ok(_) => break,
                Err(err) => {
                    info!("join failed with status={}", err.status);
                }
            }
        }
    }

    pub async fn set_led(&mut self, value: bool) {
        self.gpio_set(0, value).await
    }

    pub async fn gpio_set(&mut self, pin: u8, value: bool) {
        self.control.gpio_set(pin, value).await;
    }
}
