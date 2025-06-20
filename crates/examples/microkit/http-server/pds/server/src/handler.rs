//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::boxed::Box;
use alloc::sync::Arc;
use core::future::Future;
use core::pin::Pin;
use core::time::Duration;

use futures::future::LocalBoxFuture;
use one_shot_mutex::sync::OneShotMutex;
use smoltcp::iface::Config;
use smoltcp::time::Instant as SmoltcpInstant;

use sel4_abstract_allocator::basic::BasicAllocator;
use sel4_abstract_allocator::WithAlignmentBound;
use sel4_async_block_io::{access::ReadOnly, constant_block_sizes::BlockSize512};
use sel4_async_network::{DhcpOverrides, ManagedInterface};
use sel4_async_single_threaded_executor::{LocalPool, LocalSpawner};
use sel4_async_time::{Instant, TimerManager};
use sel4_driver_interfaces::timer::{Clock, DefaultTimer, Timer};
use sel4_microkit::{ChannelSet, Handler, Infallible};
use sel4_microkit_driver_adapters::timer::client::Client as TimerClient;
use sel4_shared_ring_buffer_block_io::SharedRingBufferBlockIO;

use crate::DeviceImpl;

pub(crate) enum Never {}

pub(crate) struct HandlerImpl {
    timer_driver_channel: sel4_microkit::Channel,
    net_driver_channel: sel4_microkit::Channel,
    block_driver_channel: sel4_microkit::Channel,
    timer: Arc<OneShotMutex<DefaultTimer<TimerClient>>>,
    net_device: DeviceImpl<WithAlignmentBound<BasicAllocator>>,
    shared_block_io:
        SharedRingBufferBlockIO<BlockSize512, ReadOnly, WithAlignmentBound<BasicAllocator>, fn()>,
    shared_timers: TimerManager,
    shared_network: ManagedInterface,
    local_pool: LocalPool,
    fut: LocalBoxFuture<'static, Never>,
}

impl HandlerImpl {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new<T: Future<Output = Never> + 'static>(
        timer_driver_channel: sel4_microkit::Channel,
        net_driver_channel: sel4_microkit::Channel,
        block_driver_channel: sel4_microkit::Channel,
        timer: Arc<OneShotMutex<DefaultTimer<TimerClient>>>,
        mut net_device: DeviceImpl<WithAlignmentBound<BasicAllocator>>,
        net_config: Config,
        shared_block_io: SharedRingBufferBlockIO<
            BlockSize512,
            ReadOnly,
            WithAlignmentBound<BasicAllocator>,
            fn(),
        >,
        f: impl FnOnce(TimerManager, ManagedInterface, LocalSpawner) -> T,
    ) -> Self {
        let now = Self::now_with_timer_client(&timer);
        let now_smoltcp = SmoltcpInstant::ZERO + now.since_zero().into();

        let shared_timers = TimerManager::new();

        let shared_network = ManagedInterface::new(
            net_config,
            DhcpOverrides::default(),
            &mut net_device,
            now_smoltcp,
        );

        let local_pool = LocalPool::new();
        let spawner = local_pool.spawner();

        let fut = Box::pin(f(shared_timers.clone(), shared_network.clone(), spawner));

        let mut this = Self {
            timer_driver_channel,
            net_driver_channel,
            block_driver_channel,
            timer,
            net_device,
            shared_block_io,
            shared_timers,
            shared_network,
            local_pool,
            fut,
        };

        this.react(true, true, true);

        this
    }

    fn now(&mut self) -> Instant {
        Self::now_with_timer_client(&self.timer)
    }

    fn now_with_timer_client(timer: &Arc<OneShotMutex<DefaultTimer<TimerClient>>>) -> Instant {
        Instant::new(timer.lock().get_time().unwrap())
    }

    fn set_timeout(&mut self, d: Duration) {
        self.timer.lock().set_timeout(d).unwrap()
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
            let now_smoltcp = SmoltcpInstant::ZERO + now.since_zero().into();
            let mut activity = false;
            activity |= self.shared_timers.poll(now);
            activity |= self.net_device.poll();
            activity |= self.shared_network.poll(now_smoltcp, &mut self.net_device);
            activity |= self.shared_block_io.poll().unwrap();
            if !activity {
                let delays = &[
                    self.shared_timers.poll_at().map(|absolute| absolute - now),
                    self.shared_network.poll_delay(now_smoltcp).map(Into::into),
                ];
                let mut repoll = false;
                if let Some(delay) = delays.iter().filter_map(Option::as_ref).min() {
                    if delay == &Duration::ZERO {
                        repoll = true;
                    } else {
                        self.set_timeout(*delay);
                    }
                }
                if !repoll {
                    break;
                }
            }
        }
    }
}

impl Handler for HandlerImpl {
    type Error = Infallible;

    fn notified(&mut self, channels: ChannelSet) -> Result<(), Self::Error> {
        self.react(
            channels.contains(self.timer_driver_channel),
            channels.contains(self.net_driver_channel),
            channels.contains(self.block_driver_channel),
        );
        Ok(())
    }
}
