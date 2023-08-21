use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use futures::future::LocalBoxFuture;

use smoltcp::iface::Config;
use smoltcp::time::{Duration, Instant};

use sel4_async_network::{DhcpOverrides, SharedNetwork};
use sel4_async_single_threaded_executor::{LocalPool, LocalSpawner};
use sel4_async_timers::SharedTimers;
use sel4cp_http_server_example_server_cpiofs as cpiofs;

use crate::{CpiofsBlockIOImpl, DeviceImpl, TimerClient, BLOCK_SIZE};

type CpiofsIOImpl =
    cpiofs::BlockIOAdapter<cpiofs::CachedBlockIO<CpiofsBlockIOImpl, BLOCK_SIZE>, BLOCK_SIZE>;

const BLOCK_CACHE_SIZE_IN_BLOCKS: usize = 128;

pub(crate) struct Reactor {
    net_device: DeviceImpl,
    blk_device: CpiofsBlockIOImpl,
    timer: TimerClient,
    net_driver_channel: sel4cp::Channel,
    blk_irq_channel: sel4cp::Channel,
    timer_driver_channel: sel4cp::Channel,
    shared_network: SharedNetwork,
    shared_timers: SharedTimers,
    local_pool: LocalPool,
    fut: LocalBoxFuture<'static, !>,
}

impl Reactor {
    pub(crate) fn new<T: Future<Output = !> + 'static>(
        net_config: Config,
        mut net_device: DeviceImpl,
        blk_device: CpiofsBlockIOImpl,
        mut timer: TimerClient,
        net_driver_channel: sel4cp::Channel,
        blk_irq_channel: sel4cp::Channel,
        timer_driver_channel: sel4cp::Channel,
        f: impl FnOnce(SharedNetwork, SharedTimers, CpiofsIOImpl, LocalSpawner) -> T,
    ) -> Self {
        let now = Self::now_with_timer_client(&mut timer);

        let shared_network = SharedNetwork::new(
            net_config,
            DhcpOverrides::default(),
            &mut net_device,
            now.clone(),
        );

        let shared_timers = SharedTimers::new(now.clone());

        let local_pool = LocalPool::new();
        let spawner = local_pool.spawner();

        let fut = Box::pin(f(
            shared_network.clone(),
            shared_timers.clone(),
            cpiofs::BlockIOAdapter::new(cpiofs::CachedBlockIO::new(
                blk_device.clone(),
                BLOCK_CACHE_SIZE_IN_BLOCKS,
            )),
            spawner,
        ));

        let mut this = Self {
            net_device,
            blk_device,
            timer,
            net_driver_channel,
            blk_irq_channel,
            timer_driver_channel,
            shared_network,
            shared_timers,
            local_pool,
            fut,
        };

        this.handle_net_interrupt();
        this.handle_blk_interrupt();
        this.handle_timer_interrupt();

        this.react();

        this
    }

    fn now(&mut self) -> Instant {
        Self::now_with_timer_client(&mut self.timer)
    }

    fn now_with_timer_client(timer: &mut TimerClient) -> Instant {
        Instant::from_micros(i64::try_from(timer.now()).unwrap())
    }

    fn set_timeout(&mut self, d: Duration) {
        self.timer.set_timeout(d.micros())
    }

    fn handle_net_interrupt(&mut self) {
        self.net_device.handle_notification();
    }

    fn handle_blk_interrupt(&mut self) {
        self.blk_device.ack_interrupt();
        self.blk_irq_channel.irq_ack().unwrap();
    }

    fn handle_timer_interrupt(&mut self) {}

    fn react(&mut self) {
        loop {
            let _ = self.local_pool.run_until_stalled(Pin::new(&mut self.fut));
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
    }
}

impl sel4cp::Handler for Reactor {
    type Error = !;

    fn notified(&mut self, channel: sel4cp::Channel) -> Result<(), Self::Error> {
        if channel == self.net_driver_channel {
            self.handle_net_interrupt();
        } else if channel == self.blk_irq_channel {
            self.handle_blk_interrupt();
        } else if channel == self.timer_driver_channel {
            self.handle_timer_interrupt();
        }

        self.react();

        Ok(())
    }
}
