//
// Copyright 2025, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(unused_variables)]
#![allow(dead_code)]

use core::marker::PhantomData;
use core::num::Wrapping;
use core::ptr::NonNull;
use core::slice;
use core::sync::atomic::Ordering;

use sddf_sys as sys;
use sel4_shared_memory::access::*;
use sel4_shared_memory::{SharedMemoryPtr, SharedMemoryRef, map_field};

use crate::{Config, common::*};

type Result<T> = core::result::Result<T, PeerMisbehaviorError>;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct ClientConfig(sys::serial_client_config);

unsafe impl Config for ClientConfig {
    fn is_magic_valid(&self) -> bool {
        self.0.magic == sys::SDDF_SERIAL_MAGIC
    }
}

impl ClientConfig {
    pub unsafe fn rx_queue(&self) -> Option<ConsumerQueue> {
        Some(ConsumerQueue::new(self.rx_shared()?, self.rx_data()?))
    }

    pub unsafe fn tx_queue(&self) -> Option<ProducerQueue> {
        Some(ProducerQueue::new(self.tx_shared()?, self.tx_data()?))
    }

    unsafe fn rx_shared(&self) -> Option<SharedMemoryRef<sys::serial_queue, ReadWrite>> {
        Some(SharedMemoryRef::new(NonNull::new(
            self.0.rx.queue.vaddr.cast(),
        )?))
    }

    unsafe fn tx_shared(&self) -> Option<SharedMemoryRef<sys::serial_queue, ReadWrite>> {
        Some(SharedMemoryRef::new(NonNull::new(
            self.0.tx.queue.vaddr.cast(),
        )?))
    }

    unsafe fn rx_data(&self) -> Option<SharedMemoryRef<[u8], ReadOnly>> {
        Some(SharedMemoryRef::new_read_only(NonNull::new(
            ptr_meta::from_raw_parts_mut(
                self.0.rx.data.vaddr.cast(),
                self.0.rx.data.size.try_into().unwrap(),
            ),
        )?))
    }

    unsafe fn tx_data(&self) -> Option<SharedMemoryRef<[u8], WriteOnly>> {
        Some(
            SharedMemoryRef::new(NonNull::new(ptr_meta::from_raw_parts_mut(
                self.0.tx.data.vaddr.cast(),
                self.0.tx.data.size.try_into().unwrap(),
            ))?)
            .write_only(),
        )
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DriverConfig(sys::serial_driver_config);

unsafe impl Config for DriverConfig {
    fn is_magic_valid(&self) -> bool {
        self.0.magic == sys::SDDF_SERIAL_MAGIC
    }
}

impl DriverConfig {
    pub unsafe fn rx_queue(&self) -> Option<ConsumerQueue> {
        Some(ConsumerQueue::new(self.rx_shared()?, self.rx_data()?))
    }

    pub unsafe fn tx_queue(&self) -> Option<ProducerQueue> {
        Some(ProducerQueue::new(self.tx_shared()?, self.tx_data()?))
    }

    unsafe fn rx_shared(&self) -> Option<SharedMemoryRef<sys::serial_queue, ReadWrite>> {
        Some(SharedMemoryRef::new(NonNull::new(
            self.0.rx.queue.vaddr.cast(),
        )?))
    }

    unsafe fn tx_shared(&self) -> Option<SharedMemoryRef<sys::serial_queue, ReadWrite>> {
        Some(SharedMemoryRef::new(NonNull::new(
            self.0.tx.queue.vaddr.cast(),
        )?))
    }

    unsafe fn rx_data(&self) -> Option<SharedMemoryRef<[u8], ReadOnly>> {
        Some(SharedMemoryRef::new_read_only(NonNull::new(
            ptr_meta::from_raw_parts_mut(
                self.0.rx.data.vaddr.cast(),
                self.0.rx.data.size.try_into().unwrap(),
            ),
        )?))
    }

    unsafe fn tx_data(&self) -> Option<SharedMemoryRef<[u8], WriteOnly>> {
        Some(
            SharedMemoryRef::new(NonNull::new(ptr_meta::from_raw_parts_mut(
                self.0.tx.data.vaddr.cast(),
                self.0.tx.data.size.try_into().unwrap(),
            ))?)
            .write_only(),
        )
    }

    pub fn default_baud(&self) -> u64 {
        self.0.default_baud
    }

    pub fn rx_enabled(&self) -> bool {
        self.0.rx_enabled
    }
}

// // //

pub struct ProducerQueue<'a> {
    inner: Queue<'a, Producer>,
}

impl<'a> ProducerQueue<'a> {
    pub fn new<A>(
        shared: SharedMemoryRef<'a, sys::serial_queue, ReadWrite>,
        data: SharedMemoryRef<'a, [u8], A>,
    ) -> Self
    where
        A: RestrictAccess<WriteOnly, Restricted = WriteOnly>,
    {
        Self {
            inner: Queue::new(shared, data.restrict()),
        }
    }

    pub fn is_full(&mut self) -> Result<bool> {
        self.inner.is_full()
    }

    pub fn free(&mut self) -> Result<usize> {
        self.inner.free()
    }

    pub fn enqueue_local(&mut self, c: u8) -> Result<bool> {
        self.inner.enqueue_local(c)
    }

    pub fn enqueue_many_local(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.enqueue_many_local(buf)
    }

    pub fn enqueue(&mut self, c: u8) -> Result<bool> {
        let ret = self.enqueue_local(c)?;
        self.update_shared_tail();
        Ok(ret)
    }

    pub fn enqueue_many(&mut self, buf: &[u8]) -> Result<usize> {
        let ret = self.enqueue_many_local(buf)?;
        self.update_shared_tail();
        Ok(ret)
    }

    pub fn update_shared_tail(&mut self) {
        self.inner.update_shared();
    }

    pub fn request_consumer_signal(&mut self) {
        self.inner
            .producer_signalled_mut()
            .atomic_store(1, Ordering::Release);
    }
}

pub struct ConsumerQueue<'a> {
    inner: Queue<'a, Consumer>,
}

impl<'a> ConsumerQueue<'a> {
    pub fn new<A>(
        shared: SharedMemoryRef<'a, sys::serial_queue, ReadWrite>,
        data: SharedMemoryRef<'a, [u8], A>,
    ) -> Self
    where
        A: RestrictAccess<ReadOnly, Restricted = ReadOnly>,
    {
        Self {
            inner: Queue::new(shared, data.restrict()),
        }
    }

    pub fn is_empty(&mut self) -> Result<bool> {
        self.inner.is_empty()
    }

    pub fn len(&mut self) -> Result<usize> {
        self.inner.len()
    }

    pub fn dequeue_local(&mut self) -> Result<Option<u8>> {
        self.inner.dequeue_local()
    }

    pub fn dequeue_many_local(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.dequeue_many_local(buf)
    }

    pub fn dequeue(&mut self) -> Result<Option<u8>> {
        let ret = self.dequeue_local()?;
        self.update_shared_head();
        Ok(ret)
    }

    pub fn dequeue_many(&mut self, buf: &mut [u8]) -> Result<usize> {
        let ret = self.dequeue_many_local(buf)?;
        self.update_shared_head();
        Ok(ret)
    }

    pub fn update_shared_head(&mut self) {
        self.inner.update_shared();
    }

    pub fn cancel_consumer_signal(&mut self) {
        self.inner
            .producer_signalled_mut()
            .atomic_store(0, Ordering::Release);
    }

    pub fn require_consumer_signal(&self) -> bool {
        self.inner.producer_signalled().read() != 0
    }
}

// // //

struct Queue<'a, R: QueueRole + ?Sized> {
    shared: SharedMemoryRef<'a, sys::serial_queue>,
    local_head: Wrapping<u32>,
    local_tail: Wrapping<u32>,
    data: SharedMemoryRef<'a, [u8], R::DataAccess>,
    _phantom: PhantomData<R>,
}

impl<'a, R: QueueRole> Queue<'a, R> {
    fn new(
        shared: SharedMemoryRef<'a, sys::serial_queue, ReadWrite>,
        data: SharedMemoryRef<'a, [u8], R::DataAccess>,
    ) -> Self {
        Self {
            shared,
            local_head: Wrapping(0),
            local_tail: Wrapping(0),
            data,
            _phantom: Default::default(),
        }
    }

    fn head(&mut self) -> SharedMemoryPtr<'_, u32, R::HeadAccess> {
        let ptr = self.shared.as_mut_ptr();
        map_field!(ptr.head).restrict()
    }

    fn tail(&mut self) -> SharedMemoryPtr<'_, u32, R::TailAccess> {
        let ptr = self.shared.as_mut_ptr();
        map_field!(ptr.tail).restrict()
    }

    fn producer_signalled(&self) -> SharedMemoryPtr<'_, u32, ReadOnly> {
        let ptr = self.shared.as_ptr();
        map_field!(ptr.producer_signalled)
    }

    fn producer_signalled_mut(&mut self) -> SharedMemoryPtr<'_, u32, R::ProducerSignalledAccess> {
        let ptr = self.shared.as_mut_ptr();
        map_field!(ptr.producer_signalled).restrict()
    }

    fn data(&mut self) -> SharedMemoryPtr<'_, [u8], R::DataAccess> {
        self.data.as_mut_ptr()
    }

    fn capacity(&self) -> usize {
        self.data.as_ptr().len()
    }

    fn residue(&self, index: Wrapping<u32>) -> usize {
        usize::try_from(index.0).unwrap() % self.capacity()
    }

    fn local_len(&self) -> usize {
        self.residue(self.local_tail - self.local_head)
    }

    fn local_contiguous_len(&self) -> usize {
        self.local_len()
            .min(self.capacity() - self.residue(self.local_head))
    }

    fn local_free(&self) -> usize {
        self.capacity() - self.local_len()
    }

    fn local_contiguous_free(&self) -> usize {
        self.local_free()
            .min(self.capacity() - self.residue(self.local_tail))
    }

    fn data_filled_prewrap(&mut self) -> SharedMemoryPtr<'_, [u8], R::DataAccess> {
        let start = self.residue(self.local_head);
        let n = self.local_contiguous_len();
        self.data().index(start..).index(..n)
    }

    fn data_filled_postwrap(&mut self) -> Option<SharedMemoryPtr<'_, [u8], R::DataAccess>> {
        let n = self.local_len() - self.local_contiguous_len();
        if n > 0 {
            Some(self.data().index(..n))
        } else {
            None
        }
    }

    fn data_free_prewrap(&mut self) -> SharedMemoryPtr<'_, [u8], R::DataAccess> {
        let start = self.residue(self.local_tail);
        let n = self.local_contiguous_free();
        self.data().index(start..).index(..n)
    }

    fn data_free_postwrap(&mut self) -> Option<SharedMemoryPtr<'_, [u8], R::DataAccess>> {
        let n = self.local_free() - self.local_contiguous_free();
        if n > 0 {
            Some(self.data().index(..n))
        } else {
            None
        }
    }

    // // //

    fn update_shared(&mut self) {
        R::update_shared(self)
    }

    fn update_local(&mut self) -> Result<()> {
        R::update_local(self)
    }

    fn len(&mut self) -> Result<usize> {
        self.update_local()?;
        Ok(self.local_len())
    }

    fn free(&mut self) -> Result<usize> {
        self.update_local()?;
        Ok(self.local_free())
    }

    fn is_empty(&mut self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    fn is_full(&mut self) -> Result<bool> {
        Ok(self.free()? == 0)
    }
}

impl<'a> Queue<'a, Producer> {
    fn enqueue_local(&mut self, c: u8) -> Result<bool> {
        Ok(self.enqueue_many_local(slice::from_ref(&c))? == 1)
    }

    fn enqueue_many_local(&mut self, buf: &[u8]) -> Result<usize> {
        self.update_local()?;
        let mut n = 0;
        let prewrap = self.data_free_prewrap();
        let n_prewrap = buf.len().min(prewrap.len());
        prewrap.copy_from_slice(&buf[..n_prewrap]);
        n += n_prewrap;
        let n_rem = n_prewrap - buf.len();
        if n_rem > 0 {
            if let Some(postwrap) = self.data_free_postwrap() {
                let n_postwrap = n_rem.min(postwrap.len());
                postwrap.copy_from_slice(&buf[n_prewrap..][..n_postwrap]);
                n += n_postwrap;
            }
        }
        self.local_tail += Wrapping(n.try_into().unwrap());
        Ok(n)
    }
}

impl<'a> Queue<'a, Consumer> {
    fn dequeue_local(&mut self) -> Result<Option<u8>> {
        let mut c = 0;
        Ok(if self.dequeue_many_local(slice::from_mut(&mut c))? == 1 {
            Some(c)
        } else {
            None
        })
    }

    fn dequeue_many_local(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.update_local()?;
        let mut n = 0;
        let prewrap = self.data_filled_prewrap();
        let n_prewrap = buf.len().min(prewrap.len());
        prewrap.copy_into_slice(&mut buf[..n_prewrap]);
        n += n_prewrap;
        let n_rem = n_prewrap - buf.len();
        if n_rem > 0 {
            if let Some(postwrap) = self.data_filled_postwrap() {
                let n_postwrap = n_rem.min(postwrap.len());
                postwrap.copy_into_slice(&mut buf[n_prewrap..][..n_postwrap]);
                n += n_postwrap;
            }
        }
        self.local_head += Wrapping(n.try_into().unwrap());
        Ok(n)
    }
}

// // //

enum Producer {}
enum Consumer {}

trait QueueRole {
    type HeadAccess: Access;
    type TailAccess: Access;
    type ProducerSignalledAccess: RestrictAccess<ReadOnly>;
    type DataAccess: RestrictAccess<ReadOnly>;

    fn update_local(queue: &mut Queue<Self>) -> Result<()>;

    fn update_shared(queue: &mut Queue<Self>);
}

impl QueueRole for Producer {
    type HeadAccess = ReadOnly;
    type TailAccess = WriteOnly;
    type ProducerSignalledAccess = WriteOnly;
    type DataAccess = WriteOnly;

    fn update_local(queue: &mut Queue<Self>) -> Result<()> {
        let observed_head = Wrapping(queue.head().read());
        let observed_len = queue.residue(queue.local_tail - observed_head);
        if observed_len > queue.local_len() {
            return Err(PeerMisbehaviorError::new());
        }
        queue.local_head = observed_head;
        Ok(())
    }

    fn update_shared(queue: &mut Queue<Self>) {
        let local_tail = queue.local_tail;
        queue.tail().atomic_store(local_tail.0, Ordering::Release)
    }
}

impl QueueRole for Consumer {
    type HeadAccess = WriteOnly;
    type TailAccess = ReadOnly;
    type ProducerSignalledAccess = ReadWrite;
    type DataAccess = ReadOnly;

    fn update_local(queue: &mut Queue<Self>) -> Result<()> {
        let observed_tail = Wrapping(queue.tail().read());
        let observed_len = queue.residue(observed_tail - queue.local_head);
        if observed_len < queue.local_len() {
            return Err(PeerMisbehaviorError::new());
        }
        queue.local_tail = observed_tail;
        Ok(())
    }

    fn update_shared(queue: &mut Queue<Self>) {
        let local_head = queue.local_head;
        queue.tail().atomic_store(local_head.0, Ordering::Release)
    }
}
