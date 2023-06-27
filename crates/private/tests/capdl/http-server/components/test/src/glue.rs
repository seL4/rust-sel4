use core::future::Future;

use smoltcp::iface::Config;
use smoltcp::phy::{Device, Medium};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::HardwareAddress;

use sel4_async_network::SharedNetwork;
use sel4_async_single_threaded_executor::{LocalPool, LocalSpawner};

use tests_capdl_http_server_components_test_sp804_driver::Driver as TimerDriver;

use crate::DeviceImpl;

const TIMER_IRQ_BADGE: sel4::Badge = 1 << 0;
const VIRTIO_NET_IRQ_BADGE: sel4::Badge = 1 << 1;

pub struct Glue {
    net_device: DeviceImpl,
    timer: TimerDriver,
    net_irq_handler: sel4::IRQHandler,
    timer_irq_handler: sel4::IRQHandler,
    shared_network: SharedNetwork,
}

impl Glue {
    pub fn new(
        mut net_device: DeviceImpl,
        timer: TimerDriver,
        net_irq_handler: sel4::IRQHandler,
        timer_irq_handler: sel4::IRQHandler,
    ) -> Self {
        let mut config = Config::new();
        config.random_seed = 0;
        if net_device.capabilities().medium == Medium::Ethernet {
            config.hardware_addr = Some(HardwareAddress::Ethernet(net_device.mac_address()));
        }

        let shared_network = SharedNetwork::new(config, &mut net_device);

        Self {
            net_device,
            timer,
            net_irq_handler,
            timer_irq_handler,
            shared_network,
        }
    }

    fn now(&mut self) -> Instant {
        Instant::from_micros(i64::try_from(self.timer.now().as_micros()).unwrap())
    }

    fn set_timeout(&mut self, d: Duration) {
        self.timer
            .set_timeout(core::time::Duration::from_micros(d.micros()))
    }

    fn handle_net_interrupt(&mut self) {
        self.net_device.ack_interrupt();
        self.net_irq_handler.irq_handler_ack().unwrap();
    }

    fn handle_timer_interrupt(&mut self) {
        self.timer.handle_interrupt();
        self.timer_irq_handler.irq_handler_ack().unwrap();
    }

    pub fn run<T: Future<Output = !>>(
        mut self,
        event_nfn: sel4::Notification,
        f: impl FnOnce(SharedNetwork, LocalSpawner) -> T,
    ) -> ! {
        self.handle_net_interrupt();
        self.handle_timer_interrupt();

        let mut local_pool = LocalPool::new();
        let spawner = local_pool.spawner();

        let fut = f(self.shared_network.clone(), spawner);
        futures::pin_mut!(fut);

        loop {
            loop {
                let _ = local_pool.run_until_stalled(fut.as_mut());
                let now = self.now();
                if !self
                    .shared_network
                    .inner()
                    .borrow_mut()
                    .poll(now, &mut self.net_device)
                {
                    break;
                }
            }

            let maybe_delay = {
                let now = self.now();
                self.shared_network.inner().borrow_mut().poll_delay(now)
            };
            if let Some(delay) = maybe_delay {
                self.set_timeout(delay);
            }

            let (_, badge) = event_nfn.wait();

            if badge & VIRTIO_NET_IRQ_BADGE != 0 {
                self.handle_net_interrupt();
            }
            if badge & TIMER_IRQ_BADGE != 0 {
                self.handle_timer_interrupt();
            }
        }
    }
}
