use crate::WEB_TASK_POOL_SIZE;
use core::str::FromStr;
use cyw43::NetDriver;
use defmt::info;
use embassy_net::{DhcpConfig, Runner, Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use heapless::String;
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
        let mut dhcp = DhcpConfig::default();
        dhcp.hostname = Some(String::from_str("robodog").unwrap());
        let config = embassy_net::Config::dhcpv4(dhcp);

        let mut rng = RoscRng;
        let seed = rng.next_u64();

        const SIZE: usize = WEB_TASK_POOL_SIZE + 1;
        static RESOURCES: StaticCell<StackResources<SIZE>> = StaticCell::new();
        let (stack, runner) = embassy_net::new(
            net_device,
            config,
            RESOURCES.init(StackResources::<SIZE>::new()),
            seed,
        );
        (Self { stack }, runner)
    }

    pub async fn up(&mut self) {
        info!("waiting for link to be up...");
        self.stack.wait_link_up().await;

        info!("waiting for config to be up...");
        self.stack.wait_config_up().await;
    }
}
