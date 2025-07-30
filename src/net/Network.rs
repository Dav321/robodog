use cyw43::NetDriver;
use defmt::info;
use embassy_net::{Runner, Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use embassy_time::Timer;
use static_cell::StaticCell;

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}

pub struct Network<'d> {
    pub stack: Stack<'d>,
}

impl<'d> Network<'d> {
    pub fn new(net_device: NetDriver<'d>) -> (Network<'d>, Runner<'d, NetDriver<'d>>) {
        let config = embassy_net::Config::dhcpv4(Default::default());

        let mut rng = RoscRng;
        let seed = rng.next_u64();

        static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            net_device,
            config,
            RESOURCES.init(StackResources::new()),
            seed,
        );
        (Self { stack }, runner)
    }

    pub async fn up(&mut self) {
        info!("waiting for DHCP...");
        while !self.stack.is_config_up() {
            Timer::after_millis(100).await;
        }

        info!("waiting for link up...");
        while !self.stack.is_link_up() {
            Timer::after_millis(500).await;
        }

        info!("waiting for stack to be up...");
        self.stack.wait_config_up().await;
    }
}
