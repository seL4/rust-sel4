use alloc::rc::Rc;
use core::cell::RefCell;
use core::task::Poll;

use async_unsync::semaphore::Semaphore;
use futures::prelude::*;
use virtio_drivers::{device::blk::*, transport::mmio::MmioTransport};

use sel4_async_request_statuses::RequestStatuses;
use tests_capdl_http_server_components_test_cpiofs::BlockIO;

use crate::HalImpl;

pub(crate) const BLOCK_SIZE: usize = SECTOR_SIZE;

// HACK hard-coded in virtio-drivers
const QUEUE_SIZE: usize = 4;

#[derive(Clone)]
pub(crate) struct CpiofsBlockIOImpl {
    inner: Rc<RefCell<CpiofsBlockIOImplInner>>,
}

struct CpiofsBlockIOImplInner {
    driver: VirtIOBlk<HalImpl, MmioTransport>,
    request_statuses: RequestStatuses<u16, ()>,
    queue_guard: Rc<Semaphore>,
}

impl CpiofsBlockIOImpl {
    pub(crate) fn new(virtio_blk: VirtIOBlk<HalImpl, MmioTransport>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(CpiofsBlockIOImplInner {
                driver: virtio_blk,
                request_statuses: RequestStatuses::new(),
                queue_guard: Rc::new(Semaphore::new(QUEUE_SIZE)),
            })),
        }
    }

    pub(crate) fn ack_interrupt(&self) {
        let _ = self.inner.borrow_mut().driver.ack_interrupt();
    }

    pub(crate) fn poll(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        if let Some(token) = inner.driver.peek_used() {
            inner.request_statuses.mark_complete(&token, ()).unwrap();
            true
        } else {
            false
        }
    }
}

impl BlockIO<BLOCK_SIZE> for CpiofsBlockIOImpl {
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]) {
        let sem = self.inner.borrow().queue_guard.clone();
        let permit = sem.acquire().await;
        let mut req = BlkReq::default();
        let mut resp = BlkResp::default();
        let token = unsafe {
            self.inner
                .borrow_mut()
                .driver
                .read_block_nb(block_id, &mut req, buf, &mut resp)
                .unwrap()
        };
        self.inner.borrow_mut().request_statuses.add(token).unwrap();
        future::poll_fn(|cx| {
            let mut inner = self.inner.borrow_mut();
            inner
                .request_statuses
                .poll(&token, cx.waker())
                .unwrap()
                .ready()?;
            unsafe {
                inner
                    .driver
                    .complete_read_block(token, &req, buf, &mut resp)
                    .unwrap();
            }
            Poll::Ready(())
        })
        .await;
        drop(permit); // is this necessary?
    }
}
