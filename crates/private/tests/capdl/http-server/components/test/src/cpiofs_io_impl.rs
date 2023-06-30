use alloc::rc::Rc;
use core::cell::RefCell;
use core::task::Poll;

use async_unsync::semaphore::Semaphore;
use futures::prelude::*;
use virtio_drivers::{device::blk::*, transport::mmio::MmioTransport};

use tests_capdl_http_server_components_test_cpiofs::BlockIO;

use crate::{HalImpl, Requests};

pub const BLOCK_SIZE: usize = SECTOR_SIZE;

// HACK hard-coded in virtio-drivers
const QUEUE_SIZE: usize = 4;

#[derive(Clone)]
pub struct CpiofsBlockIOImpl {
    pub inner: Rc<RefCell<CpiofsBlockIOImplInner>>,
}

pub struct CpiofsBlockIOImplInner {
    driver: VirtIOBlk<HalImpl, MmioTransport>,
    requests: Requests<u16, ()>,
    queue_guard: Rc<Semaphore>,
}

impl CpiofsBlockIOImpl {
    pub fn new(virtio_blk: VirtIOBlk<HalImpl, MmioTransport>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(CpiofsBlockIOImplInner {
                driver: virtio_blk,
                requests: Requests::new(),
                queue_guard: Rc::new(Semaphore::new(QUEUE_SIZE)),
            })),
        }
    }

    pub fn ack_interrupt(&self) {
        let _ = self.inner.borrow_mut().driver.ack_interrupt();
    }

    pub fn poll(&self) -> bool {
        let mut inner = self.inner.borrow_mut();
        if let Some(token) = inner.driver.peek_used() {
            inner.requests.mark_complete(&token, ());
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
        self.inner.borrow_mut().requests.add(token);
        future::poll_fn(|cx| {
            let mut inner = self.inner.borrow_mut();
            inner.requests.poll(&token, cx.waker()).ready()?;
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
