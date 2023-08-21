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

pub(crate) struct HandlerImpl {
    timer_driver_channel: sel4cp::Channel,
    net_driver_channel: sel4cp::Channel,
    virtio_blk_irq_channel: sel4cp::Channel,
    timer: TimerClient,
    net_device: DeviceImpl,
    fs_block_io: CpiofsBlockIOImpl,
    shared_timers: SharedTimers,
    shared_network: SharedNetwork,
    local_pool: LocalPool,
    fut: LocalBoxFuture<'static, !>,
}

impl HandlerImpl {
    pub(crate) fn new<T: Future<Output = !> + 'static>(
        timer_driver_channel: sel4cp::Channel,
        net_driver_channel: sel4cp::Channel,
        virtio_blk_irq_channel: sel4cp::Channel,
        mut timer: TimerClient,
        mut net_device: DeviceImpl,
        net_config: Config,
        fs_block_io: CpiofsBlockIOImpl,
        f: impl FnOnce(SharedTimers, SharedNetwork, CpiofsIOImpl, LocalSpawner) -> T,
    ) -> Self {
        let now = Self::now_with_timer_client(&mut timer);

        let shared_timers = SharedTimers::new(now.clone());

        let shared_network = SharedNetwork::new(
            net_config,
            DhcpOverrides::default(),
            &mut net_device,
            now.clone(),
        );

        let local_pool = LocalPool::new();
        let spawner = local_pool.spawner();

        let fs_io = cpiofs::BlockIOAdapter::new(cpiofs::CachedBlockIO::new(
            fs_block_io.clone(),
            BLOCK_CACHE_SIZE_IN_BLOCKS,
        ));

        let fut = Box::pin(f(
            shared_timers.clone(),
            shared_network.clone(),
            fs_io,
            spawner,
        ));

        let mut this = Self {
            timer_driver_channel,
            net_driver_channel,
            virtio_blk_irq_channel,
            timer,
            net_device,
            fs_block_io,
            shared_timers,
            shared_network,
            local_pool,
            fut,
        };

        this.handle_timer_notification();
        this.handle_net_notification();
        this.handle_virtio_blk_interrupt();

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

    fn handle_timer_notification(&mut self) {}

    fn handle_net_notification(&mut self) {
        self.net_device.handle_notification();
    }

    fn handle_virtio_blk_interrupt(&mut self) {
        self.fs_block_io.ack_interrupt();
        self.virtio_blk_irq_channel.irq_ack().unwrap();
    }

    fn react(&mut self) {
        loop {
            let _ = self.local_pool.run_until_stalled(Pin::new(&mut self.fut));
            let now = self.now();
            let mut activity = false;
            activity |= self.shared_timers.inner().borrow_mut().poll(now);
            activity |= self
                .shared_network
                .inner()
                .borrow_mut()
                .poll(now, &mut self.net_device);
            activity |= self.fs_block_io.poll();
            if !activity {
                let delays = &[
                    self.shared_timers.inner().borrow_mut().poll_delay(now),
                    self.shared_network.inner().borrow_mut().poll_delay(now),
                ];
                if let Some(delay) = delays.iter().filter_map(Option::as_ref).min() {
                    self.set_timeout(delay.clone());
                }
                break;
            }
        }
    }
}

impl sel4cp::Handler for HandlerImpl {
    type Error = !;

    fn notified(&mut self, channel: sel4cp::Channel) -> Result<(), Self::Error> {
        if channel == self.timer_driver_channel {
            self.handle_timer_notification();
        } else if channel == self.net_driver_channel {
            self.handle_net_notification();
        } else if channel == self.virtio_blk_irq_channel {
            self.handle_virtio_blk_interrupt();
        }

        self.react();

        Ok(())
    }
}
