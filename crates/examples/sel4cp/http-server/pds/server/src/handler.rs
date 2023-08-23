use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;

use futures::future::LocalBoxFuture;

use smoltcp::iface::Config;
use smoltcp::time::{Duration, Instant};

use sel4_async_block_io::{BytesIOAdapter, CachedBlockIO};
use sel4_async_network::{DhcpOverrides, SharedNetwork};
use sel4_async_single_threaded_executor::{LocalPool, LocalSpawner};
use sel4_async_timers::SharedTimers;
use sel4_shared_ring_buffer_block_io::BlockIO;

use crate::{DeviceImpl, TimerClient, BLOCK_SIZE};

type BytesIOImpl = BytesIOAdapter<CachedBlockIO<BlockIO, BLOCK_SIZE>, BLOCK_SIZE>;

const BLOCK_CACHE_SIZE_IN_BLOCKS: usize = 128;

pub(crate) struct HandlerImpl {
    timer_driver_channel: sel4cp::Channel,
    net_driver_channel: sel4cp::Channel,
    block_driver_channel: sel4cp::Channel,
    timer: TimerClient,
    net_device: DeviceImpl,
    fs_block_io: BlockIO,
    shared_timers: SharedTimers,
    shared_network: SharedNetwork,
    local_pool: LocalPool,
    fut: LocalBoxFuture<'static, !>,
}

impl HandlerImpl {
    pub(crate) fn new<T: Future<Output = !> + 'static>(
        timer_driver_channel: sel4cp::Channel,
        net_driver_channel: sel4cp::Channel,
        block_driver_channel: sel4cp::Channel,
        mut timer: TimerClient,
        mut net_device: DeviceImpl,
        net_config: Config,
        fs_block_io: BlockIO,
        f: impl FnOnce(SharedTimers, SharedNetwork, BytesIOImpl, LocalSpawner) -> T,
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

        let fs_io = BytesIOAdapter::new(CachedBlockIO::new(
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
            block_driver_channel,
            timer,
            net_device,
            fs_block_io,
            shared_timers,
            shared_network,
            local_pool,
            fut,
        };

        this.react(true, true, true);

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

    // TODO focused polling using these args doesn't play nicely with "repoll" mechanism below
    fn react(
        &mut self,
        _timer_notification: bool,
        _net_notification: bool,
        _block_notification: bool,
    ) {
        loop {
            let _ = self.local_pool.run_until_stalled(Pin::new(&mut self.fut));
            let now = self.now();
            let mut activity = false;
            activity |= self.shared_timers.poll(now);
            activity |= self.net_device.poll();
            activity |= self.shared_network.poll(now, &mut self.net_device);
            activity |= self.fs_block_io.poll();
            if !activity {
                let delays = &[
                    self.shared_timers.poll_delay(now),
                    self.shared_network.poll_delay(now),
                ];
                let mut repoll = false;
                if let Some(delay) = delays.iter().filter_map(Option::as_ref).min() {
                    if delay == &Duration::ZERO {
                        repoll = true;
                    } else {
                        self.set_timeout(delay.clone());
                    }
                }
                if !repoll {
                    break;
                }
            }
        }
    }
}

impl sel4cp::Handler for HandlerImpl {
    type Error = !;

    fn notified(&mut self, channel: sel4cp::Channel) -> Result<(), Self::Error> {
        self.react(
            channel == self.timer_driver_channel,
            channel == self.net_driver_channel,
            channel == self.block_driver_channel,
        );
        Ok(())
    }
}
