use core::future::Future;

use smoltcp::iface::Config;
use smoltcp::phy::{Device, Medium};
use smoltcp::time::{Duration, Instant};
use smoltcp::wire::HardwareAddress;

use sel4_async_network::{DhcpOverrides, SharedNetwork};
use sel4_async_single_threaded_executor::{LocalPool, LocalSpawner};
use sel4_async_timers::SharedTimers;
use tests_capdl_http_server_components_http_server_cpiofs as cpiofs;
use tests_capdl_http_server_components_timer_driver_sp804_driver::Driver as TimerDriver;

use crate::{CpiofsBlockIOImpl, DeviceImpl, BLOCK_SIZE};

const TIMER_IRQ_BADGE: sel4::Badge = 1 << 0;
const VIRTIO_NET_IRQ_BADGE: sel4::Badge = 1 << 1;
const VIRTIO_BLK_IRQ_BADGE: sel4::Badge = 1 << 2;

type CpiofsIOImpl =
    cpiofs::BlockIOAdapter<cpiofs::CachedBlockIO<CpiofsBlockIOImpl, BLOCK_SIZE>, BLOCK_SIZE>;

const BLOCK_CACHE_SIZE_IN_BLOCKS: usize = 128;

pub(crate) struct Reactor {
    net_device: DeviceImpl,
    blk_device: CpiofsBlockIOImpl,
    timer: TimerDriver,
    net_irq_handler: sel4::IRQHandler,
    blk_irq_handler: sel4::IRQHandler,
    timer_irq_handler: sel4::IRQHandler,
    shared_network: SharedNetwork,
    shared_timers: SharedTimers,
}

impl Reactor {
    pub(crate) fn new(
        mut net_device: DeviceImpl,
        blk_device: CpiofsBlockIOImpl,
        mut timer: TimerDriver,
        net_irq_handler: sel4::IRQHandler,
        blk_irq_handler: sel4::IRQHandler,
        timer_irq_handler: sel4::IRQHandler,
    ) -> Self {
        assert_eq!(net_device.capabilities().medium, Medium::Ethernet);
        let hardware_addr = HardwareAddress::Ethernet(net_device.mac_address());
        let mut config = Config::new(hardware_addr);
        config.random_seed = 0;

        let now = Self::now_with_timer_driver(&mut timer);

        let shared_network = SharedNetwork::new(
            config,
            DhcpOverrides::default(),
            &mut net_device,
            now.clone(),
        );

        let shared_timers = SharedTimers::new(now.clone());

        Self {
            net_device,
            blk_device,
            timer,
            net_irq_handler,
            blk_irq_handler,
            timer_irq_handler,
            shared_network,
            shared_timers,
        }
    }

    fn now(&mut self) -> Instant {
        Self::now_with_timer_driver(&mut self.timer)
    }

    fn now_with_timer_driver(timer: &mut TimerDriver) -> Instant {
        Instant::from_micros(i64::try_from(timer.now().as_micros()).unwrap())
    }

    fn set_timeout(&mut self, d: Duration) {
        self.timer
            .set_timeout(core::time::Duration::from_micros(d.micros()))
    }

    fn handle_net_interrupt(&mut self) {
        self.net_device.ack_interrupt();
        self.net_irq_handler.irq_handler_ack().unwrap();
    }

    fn handle_blk_interrupt(&mut self) {
        self.blk_device.ack_interrupt();
        self.blk_irq_handler.irq_handler_ack().unwrap();
    }

    fn handle_timer_interrupt(&mut self) {
        self.timer.handle_interrupt();
        self.timer_irq_handler.irq_handler_ack().unwrap();
    }

    pub(crate) fn run<T: Future<Output = !>>(
        mut self,
        event_nfn: sel4::Notification,
        f: impl FnOnce(SharedNetwork, SharedTimers, CpiofsIOImpl, LocalSpawner) -> T,
    ) -> ! {
        self.handle_net_interrupt();
        self.handle_blk_interrupt();
        self.handle_timer_interrupt();

        let mut local_pool = LocalPool::new();
        let spawner = local_pool.spawner();

        let fut = f(
            self.shared_network.clone(),
            self.shared_timers.clone(),
            cpiofs::BlockIOAdapter::new(cpiofs::CachedBlockIO::new(
                self.blk_device.clone(),
                BLOCK_CACHE_SIZE_IN_BLOCKS,
            )),
            spawner,
        );
        futures::pin_mut!(fut);

        loop {
            loop {
                let _ = local_pool.run_until_stalled(fut.as_mut());
                let now = self.now();
                let mut activity = false;
                activity |= self.blk_device.poll();
                activity |= self
                    .shared_network
                    .inner()
                    .borrow_mut()
                    .poll(now, &mut self.net_device);
                activity |= self.shared_timers.inner().borrow_mut().poll(now);
                if !activity {
                    let delays = &[
                        self.shared_network.inner().borrow_mut().poll_delay(now),
                        self.shared_timers.inner().borrow_mut().poll_delay(now),
                    ];
                    if let Some(delay) = delays.iter().filter_map(Option::as_ref).min() {
                        self.set_timeout(delay.clone());
                    }
                    break;
                }
            }

            let (_, badge) = event_nfn.wait();

            if badge & VIRTIO_NET_IRQ_BADGE != 0 {
                self.handle_net_interrupt();
            }
            if badge & VIRTIO_BLK_IRQ_BADGE != 0 {
                self.handle_blk_interrupt();
            }
            if badge & TIMER_IRQ_BADGE != 0 {
                self.handle_timer_interrupt();
            }
        }
    }
}
