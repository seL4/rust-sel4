use core::ptr::NonNull;

use virtio_drivers::{
    device::net::*,
    transport::{
        mmio::{MmioTransport, VirtIOHeader},
        DeviceType, Transport,
    },
};

use crate::Config;

mod hal;

pub use hal::HalImpl;

const NET_BUFFER_LEN: usize = 2048;
const NET_QUEUE_SIZE: usize = 16;

pub fn init(config: &Config) -> VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE> {
    HalImpl::init(
        NonNull::slice_from_raw_parts(
            NonNull::new(config.virtio_net_dma_vaddr_range.start as *mut _).unwrap(),
            config.virtio_net_dma_vaddr_range.end - config.virtio_net_dma_vaddr_range.start,
        ),
        config.virtio_net_dma_vaddr_to_paddr_offset,
    );

    {
        let header = NonNull::new(
            (config.virtio_net_mmio_vaddr + config.virtio_net_mmio_offset) as *mut VirtIOHeader,
        )
        .unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        assert_eq!(transport.device_type(), DeviceType::Network);
        VirtIONet::new(transport, NET_BUFFER_LEN).unwrap()
    }
}
