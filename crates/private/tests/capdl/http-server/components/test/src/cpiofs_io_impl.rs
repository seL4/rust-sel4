use alloc::collections::{btree_map, BTreeMap};
use alloc::rc::Rc;
use core::cell::RefCell;
use core::task::{Poll, Waker};

use async_unsync::semaphore::Semaphore;
use futures::prelude::*;
use virtio_drivers::{device::blk::*, transport::mmio::MmioTransport};

use tests_capdl_http_server_components_test_cpiofs::IO;

use crate::HalImpl;

// HACK hard-coded in virtio-drivers
const QUEUE_SIZE: usize = 4;

#[derive(Clone)]
pub struct CpiofsIOImpl {
    pub inner: Rc<RefCell<CpiofsIOImplInner>>,
}

pub struct CpiofsIOImplInner {
    driver: VirtIOBlk<HalImpl, MmioTransport>,
    pending: BTreeMap<u16, Option<Waker>>,
    queue_guard: Rc<Semaphore>,
}

impl CpiofsIOImpl {
    pub fn new(virtio_blk: VirtIOBlk<HalImpl, MmioTransport>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(CpiofsIOImplInner {
                driver: virtio_blk,
                pending: BTreeMap::new(),
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
            if let Some(pending) = inner.pending.remove(&token) {
                if let Some(waker) = pending {
                    waker.wake();
                    return true;
                } else {
                    log::warn!("token={} had no waker", token);
                }
            } else {
                log::warn!("token={} was not pending", token);
            }
        }
        false
    }

    pub async fn read_block(&self, block_id: usize, buf: &mut [u8; SECTOR_SIZE]) {
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
        self.inner.borrow_mut().pending.insert(token, None);
        future::poll_fn(|cx| {
            let mut inner = self.inner.borrow_mut();
            match inner.pending.entry(token) {
                btree_map::Entry::Vacant(_) => {
                    unsafe {
                        inner
                            .driver
                            .complete_read_block(token, &req, buf, &mut resp)
                            .unwrap();
                    }
                    Poll::Ready(())
                }
                btree_map::Entry::Occupied(mut occupied) => {
                    occupied.insert(Some(cx.waker().clone()));
                    Poll::Pending
                }
            }
        })
        .await;
        drop(permit); // is this necessary?
    }
}

impl IO for CpiofsIOImpl {
    async fn read(&self, offset: usize, buf: &mut [u8]) {
        let mut block_buf = [0; SECTOR_SIZE];
        let start_offset = offset;
        let end_offset = offset + buf.len();
        let start_block_id = start_offset / SECTOR_SIZE;
        let end_block_id = end_offset.next_multiple_of(SECTOR_SIZE) / SECTOR_SIZE;
        for block_id in start_block_id..end_block_id {
            self.read_block(block_id, &mut block_buf).await;
            let this_start_offset = start_offset.max(block_id * SECTOR_SIZE);
            let this_end_offset = end_offset.min((block_id + 1) * SECTOR_SIZE);
            let this_len = this_end_offset - this_start_offset;
            buf[this_start_offset - start_offset..this_end_offset - start_offset]
                .copy_from_slice(&block_buf[this_start_offset % SECTOR_SIZE..][..this_len]);
        }
    }
}
