//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use core::marker::PhantomData;
use core::num::Wrapping;
use core::sync::atomic::{AtomicU32, Ordering};

use zerocopy::{FromBytes, IntoBytes};

use sel4_shared_memory::{SharedMemoryPtr, SharedMemoryRef, map_field};

pub mod roles;

use roles::{Read, RingBufferRole, RingBufferRoleValue, RingBuffersRole, Write};

mod descriptor;

pub use descriptor::Descriptor;

// TODO
// - zerocopy for RawRingBuffer
// - require zerocopy for T in enqueue and dequeue?
// - variable length descriptor array?

pub const RING_BUFFER_SIZE: usize = 512;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct PeerMisbehaviorError(());

impl PeerMisbehaviorError {
    fn new() -> Self {
        Self(())
    }
}

pub struct RingBuffers<'a, R: RingBuffersRole, F, T = Descriptor> {
    free: RingBuffer<'a, R::FreeRole, T>,
    used: RingBuffer<'a, R::UsedRole, T>,
    notify: F,
}

impl<'a, R: RingBuffersRole, F, T: Copy> RingBuffers<'a, R, F, T> {
    pub fn new(
        free: RingBuffer<'a, R::FreeRole, T>,
        used: RingBuffer<'a, R::UsedRole, T>,
        notify: F,
    ) -> Self {
        Self { free, used, notify }
    }

    pub fn from_ptrs_using_default_initialization_strategy_for_role(
        free: SharedMemoryRef<'a, RawRingBuffer<T>>,
        used: SharedMemoryRef<'a, RawRingBuffer<T>>,
        notify: F,
    ) -> Self {
        let initialization_strategy = R::default_initialization_strategy();
        Self::new(
            RingBuffer::new(free, initialization_strategy),
            RingBuffer::new(used, initialization_strategy),
            notify,
        )
    }

    pub fn free(&self) -> &RingBuffer<'a, R::FreeRole, T> {
        &self.free
    }

    pub fn used(&self) -> &RingBuffer<'a, R::UsedRole, T> {
        &self.used
    }

    pub fn free_mut(&mut self) -> &mut RingBuffer<'a, R::FreeRole, T> {
        &mut self.free
    }

    pub fn used_mut(&mut self) -> &mut RingBuffer<'a, R::UsedRole, T> {
        &mut self.used
    }
}

impl<U, R: RingBuffersRole, F: Fn() -> U, T> RingBuffers<'_, R, F, T> {
    pub fn notify(&self) -> U {
        (self.notify)()
    }
}

impl<U, R: RingBuffersRole, F: FnMut() -> U, T> RingBuffers<'_, R, F, T> {
    pub fn notify_mut(&mut self) -> U {
        (self.notify)()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct RawRingBuffer<T = Descriptor> {
    pub write_index: AtomicU32,
    pub read_index: AtomicU32,
    pub descriptors: [T; RING_BUFFER_SIZE],
}

pub struct RingBuffer<'a, R: RingBufferRole, T = Descriptor> {
    inner: SharedMemoryRef<'a, RawRingBuffer<T>>,
    stored_write_index: Wrapping<u32>,
    stored_read_index: Wrapping<u32>,
    _phantom: PhantomData<R>,
}

impl<'a, R: RingBufferRole, T: Copy> RingBuffer<'a, R, T> {
    const SIZE: usize = RING_BUFFER_SIZE;

    pub fn new(
        ptr: SharedMemoryRef<'a, RawRingBuffer<T>>,
        initialization_strategy: InitializationStrategy,
    ) -> Self {
        let mut inner = ptr;
        let initial_state = match initialization_strategy {
            InitializationStrategy::ReadState => {
                let ptr = inner.as_ptr();
                InitialState {
                    write_index: map_field!(ptr.write_index).read().into_inner(),
                    read_index: map_field!(ptr.read_index).read().into_inner(),
                }
            }
            InitializationStrategy::UseState(initial_state) => initial_state,
            InitializationStrategy::UseAndWriteState(initial_state) => {
                let ptr = inner.as_mut_ptr();
                map_field!(ptr.write_index).write(initial_state.write_index.into());
                map_field!(ptr.read_index).write(initial_state.read_index.into());
                initial_state
            }
        };
        Self {
            inner,
            stored_write_index: Wrapping(initial_state.write_index),
            stored_read_index: Wrapping(initial_state.read_index),
            _phantom: PhantomData,
        }
    }

    const fn role(&self) -> RingBufferRoleValue {
        R::ROLE
    }

    pub const fn capacity(&self) -> usize {
        Self::SIZE - 1
    }

    fn write_index(&mut self) -> SharedMemoryPtr<'_, AtomicU32> {
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.write_index)
    }

    fn read_index(&mut self) -> SharedMemoryPtr<'_, AtomicU32> {
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.read_index)
    }

    fn descriptor(&mut self, index: Wrapping<u32>) -> SharedMemoryPtr<'_, T> {
        let residue = self.residue(index);
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.descriptors).as_slice().index(residue)
    }

    fn update_stored_write_index(&mut self) -> Result<(), PeerMisbehaviorError> {
        debug_assert!(self.role().is_read());
        let observed_write_index = Wrapping(self.write_index().read().into_inner());
        let observed_num_filled_slots = self.residue(observed_write_index - self.stored_read_index);
        if observed_num_filled_slots < self.stored_num_filled_slots() {
            return Err(PeerMisbehaviorError::new());
        }
        self.stored_write_index = observed_write_index;
        Ok(())
    }

    fn update_stored_read_index(&mut self) -> Result<(), PeerMisbehaviorError> {
        debug_assert!(self.role().is_write());
        let observed_read_index = Wrapping(self.read_index().read().into_inner());
        let observed_num_filled_slots = self.residue(self.stored_write_index - observed_read_index);
        if observed_num_filled_slots > self.stored_num_filled_slots() {
            return Err(PeerMisbehaviorError::new());
        }
        self.stored_read_index = observed_read_index;
        Ok(())
    }

    fn stored_num_filled_slots(&mut self) -> usize {
        self.residue(self.stored_write_index - self.stored_read_index)
    }

    pub fn num_filled_slots(&mut self) -> Result<usize, PeerMisbehaviorError> {
        match self.role() {
            RingBufferRoleValue::Read => self.update_stored_write_index(),
            RingBufferRoleValue::Write => self.update_stored_read_index(),
        }?;
        Ok(self.stored_num_filled_slots())
    }

    pub fn num_empty_slots(&mut self) -> Result<usize, PeerMisbehaviorError> {
        Ok(self.capacity() - self.num_filled_slots()?)
    }

    pub fn is_empty(&mut self) -> Result<bool, PeerMisbehaviorError> {
        Ok(self.num_filled_slots()? == 0)
    }

    pub fn is_full(&mut self) -> Result<bool, PeerMisbehaviorError> {
        Ok(self.num_empty_slots()? == 0)
    }

    fn residue(&self, index: Wrapping<u32>) -> usize {
        usize::try_from(index.0).unwrap() % Self::SIZE
    }
}

impl<T: Copy + FromBytes + IntoBytes> RingBuffer<'_, Write, T> {
    pub fn enqueue_and_commit(&mut self, desc: T) -> Result<Result<(), T>, PeerMisbehaviorError> {
        self.enqueue(desc, true)
    }

    pub fn enqueue_without_committing(
        &mut self,
        desc: T,
    ) -> Result<Result<(), T>, PeerMisbehaviorError> {
        self.enqueue(desc, false)
    }

    pub fn enqueue(
        &mut self,
        desc: T,
        commit: bool,
    ) -> Result<Result<(), T>, PeerMisbehaviorError> {
        if self.is_full()? {
            return Ok(Err(desc));
        }
        self.force_enqueue(desc, commit);
        Ok(Ok(()))
    }

    pub fn force_enqueue(&mut self, desc: T, commit: bool) {
        self.descriptor(self.stored_write_index).write(desc);
        self.stored_write_index += 1;
        if commit {
            self.commit();
        }
    }

    pub fn commit(&mut self) {
        self.expose_write_index();
    }

    fn expose_write_index(&mut self) {
        let write_index = self.stored_write_index.0;
        self.write_index()
            .atomic_store(write_index, Ordering::Release);
    }
}

impl<T: Copy + FromBytes + IntoBytes> RingBuffer<'_, Read, T> {
    pub fn dequeue(&mut self) -> Result<Option<T>, PeerMisbehaviorError> {
        if self.is_empty()? {
            return Ok(None);
        }
        Ok(Some(self.force_dequeue()))
    }

    pub fn force_dequeue(&mut self) -> T {
        let desc = self.descriptor(self.stored_read_index).read();
        self.stored_read_index += 1;
        self.expose_read_index();
        desc
    }

    fn expose_read_index(&mut self) {
        let read_index = self.stored_read_index.0;
        self.read_index()
            .atomic_store(read_index, Ordering::Release);
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub enum InitializationStrategy {
    #[default]
    ReadState,
    UseState(InitialState),
    UseAndWriteState(InitialState),
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Default)]
pub struct InitialState {
    write_index: u32,
    read_index: u32,
}

impl InitialState {
    pub fn new(write_index: u32, read_index: u32) -> Self {
        Self {
            write_index,
            read_index,
        }
    }
}
