#![no_std]

use core::num::Wrapping;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;

use zerocopy::{AsBytes, FromBytes, FromZeroes};

use sel4_externally_shared::{map_field, ExternallySharedPtr, ExternallySharedRef};

pub const RING_BUFFER_SIZE: usize = 512;

pub struct RingBuffers<'a, F, T = Descriptor> {
    free: RingBuffer<'a, T>,
    used: RingBuffer<'a, T>,
    notify: F,
}

impl<'a, F, T: Copy> RingBuffers<'a, F, T> {
    pub fn new(
        free: RingBuffer<'a, T>,
        used: RingBuffer<'a, T>,
        notify: F,
        initialize: bool,
    ) -> Self {
        let mut this = Self { free, used, notify };
        if initialize {
            this.free_mut().initialize();
            this.used_mut().initialize();
        }
        this
    }

    pub fn free(&self) -> &RingBuffer<'a, T> {
        &self.free
    }

    pub fn used(&self) -> &RingBuffer<'a, T> {
        &self.used
    }

    pub fn free_mut(&mut self) -> &mut RingBuffer<'a, T> {
        &mut self.free
    }

    pub fn used_mut(&mut self) -> &mut RingBuffer<'a, T> {
        &mut self.used
    }
}

impl<'a, T, F: Fn() -> R, R> RingBuffers<'a, F, T> {
    pub fn notify(&self) -> R {
        (self.notify)()
    }
}

impl<'a, T, F: FnMut() -> R, R> RingBuffers<'a, F, T> {
    pub fn notify_mut(&mut self) -> R {
        (self.notify)()
    }
}

// TODO: zerocopy AsBytes and FromBytes
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct RawRingBuffer<T = Descriptor> {
    write_index: u32,
    read_index: u32,
    descriptors: [T; RING_BUFFER_SIZE],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, AsBytes, FromBytes, FromZeroes)]
pub struct Descriptor {
    encoded_addr: usize,
    len: u32,
    _padding: [u8; 4],
    cookie: usize,
}

impl Descriptor {
    pub fn new(encoded_addr: usize, len: u32, cookie: usize) -> Self {
        Self {
            encoded_addr,
            len,
            _padding: [0; 4],
            cookie,
        }
    }

    pub fn encoded_addr(&self) -> usize {
        self.encoded_addr
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn cookie(&self) -> usize {
        self.cookie
    }
}

pub struct RingBuffer<'a, T = Descriptor> {
    inner: ExternallySharedRef<'a, RawRingBuffer<T>>,
}

impl<'a, T: Copy> RingBuffer<'a, T> {
    // TODO parameterizing RingBuffer to use this const is not very ergonomic
    // pub const SIZE: usize = RING_BUFFER_SIZE;

    pub fn new(inner: ExternallySharedRef<'a, RawRingBuffer<T>>) -> Self {
        Self { inner }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_ptr(ptr: NonNull<RawRingBuffer<T>>) -> Self {
        Self::new(ExternallySharedRef::new(ptr))
    }

    fn write_index(&self) -> Wrapping<u32> {
        let ptr = self.inner.as_ptr();
        Wrapping(map_field!(ptr.write_index).read())
    }

    fn read_index(&self) -> Wrapping<u32> {
        let ptr = self.inner.as_ptr();
        Wrapping(map_field!(ptr.read_index).read())
    }

    fn set_write_index(&mut self, index: Wrapping<u32>) {
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.write_index).write(index.0)
    }

    fn set_read_index(&mut self, index: Wrapping<u32>) {
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.read_index).write(index.0)
    }

    fn initialize(&mut self) {
        self.set_write_index(Wrapping(0));
        self.set_read_index(Wrapping(0));
    }

    fn descriptor(&mut self, index: Wrapping<u32>) -> ExternallySharedPtr<'_, T> {
        let linear_index = usize::try_from(residue(index).0).unwrap();
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.descriptors).as_slice().index(linear_index)
    }

    pub fn is_empty(&self) -> bool {
        !has_nonzero_residue(self.write_index() - self.read_index())
    }

    pub fn is_full(&self) -> bool {
        !has_nonzero_residue(self.write_index() - self.read_index() + Wrapping(1))
    }

    pub fn enqueue(&mut self, desc: T) -> Result<(), Error> {
        if self.is_full() {
            return Err(Error::RingIsFull);
        }
        self.descriptor(self.write_index()).write(desc);
        {
            let ptr = self.inner.as_mut_ptr();
            map_field!(ptr.write_index).with_atomic(|x| x.fetch_add(1, Ordering::Release));
        }
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<T, Error> {
        if self.is_empty() {
            return Err(Error::RingIsEmpty);
        }
        let desc = self.descriptor(self.read_index()).read();
        {
            let ptr = self.inner.as_mut_ptr();
            map_field!(ptr.read_index).with_atomic(|x| x.fetch_add(1, Ordering::Release));
        }
        Ok(desc)
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Error {
    RingIsFull,
    RingIsEmpty,
}

fn residue(n: Wrapping<u32>) -> Wrapping<u32> {
    let size = Wrapping(u32::try_from(RING_BUFFER_SIZE).unwrap());
    n % size
}

fn has_nonzero_residue(n: Wrapping<u32>) -> bool {
    residue(n) != Wrapping(0)
}
