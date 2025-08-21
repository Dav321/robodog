use crate::WEB_TASK_POOL_SIZE;
use cyw43::NetDriver;
use defmt::info;
use embassy_net::{Ipv4Address, Ipv4Cidr, Runner, Stack, StackResources, StaticConfigV4};
use embassy_rp::clocks::RoscRng;
use heapless::Vec;
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
        let config = embassy_net::Config::ipv4_static(
            StaticConfigV4 {
                address: Ipv4Cidr::new(Ipv4Address::new(169, 254, 1, 1), 16),
                gateway: None,
                dns_servers: Vec::<Ipv4Address, 3>::new(),
            }
        );

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
        info!("waiting for config to be up...");
        self.stack.wait_config_up().await;
    }
}
