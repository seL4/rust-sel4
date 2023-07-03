#![no_std]

use core::num::Wrapping;
use core::ptr::NonNull;
use core::sync::atomic::Ordering;

use zerocopy::{AsBytes, FromBytes};

use sel4_externally_shared::{map_field, ExternallySharedPtr, ExternallySharedRef};

pub struct RingBuffers<'a, F> {
    free: RingBuffer<'a>,
    used: RingBuffer<'a>,
    notify: F,
}

impl<'a, F> RingBuffers<'a, F> {
    pub fn new(free: RingBuffer<'a>, used: RingBuffer<'a>, notify: F, initialize: bool) -> Self {
        let mut this = Self { free, used, notify };
        if initialize {
            this.free_mut().initialize();
            this.used_mut().initialize();
        }
        this
    }

    pub fn free(&self) -> &RingBuffer<'a> {
        &self.free
    }

    pub fn used(&self) -> &RingBuffer<'a> {
        &self.used
    }

    pub fn free_mut(&mut self) -> &mut RingBuffer<'a> {
        &mut self.free
    }

    pub fn used_mut(&mut self) -> &mut RingBuffer<'a> {
        &mut self.used
    }
}

impl<'a, F: Fn() -> R, R> RingBuffers<'a, F> {
    pub fn notify(&self) -> R {
        (self.notify)()
    }
}

impl<'a, F: FnMut() -> R, R> RingBuffers<'a, F> {
    pub fn notify_mut(&mut self) -> R {
        (self.notify)()
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
pub struct RawRingBuffer {
    write_index: u32,
    read_index: u32,
    descriptors: [Descriptor; RingBuffer::SIZE],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes, FromBytes)]
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

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn cookie(&self) -> usize {
        self.cookie
    }
}

pub struct RingBuffer<'a> {
    inner: ExternallySharedRef<'a, RawRingBuffer>,
}

impl<'a> RingBuffer<'a> {
    pub const SIZE: usize = 512;

    pub unsafe fn new(inner: ExternallySharedRef<'a, RawRingBuffer>) -> Self {
        Self { inner }
    }

    pub unsafe fn from_ptr(ptr: NonNull<RawRingBuffer>) -> Self {
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

    fn descriptor(&mut self, index: Wrapping<u32>) -> ExternallySharedPtr<'_, Descriptor> {
        let linear_index = usize::try_from(Self::residue(index).0).unwrap();
        let ptr = self.inner.as_mut_ptr();
        map_field!(ptr.descriptors).as_slice().index(linear_index)
    }

    fn residue(n: Wrapping<u32>) -> Wrapping<u32> {
        let size = Wrapping(u32::try_from(Self::SIZE).unwrap());
        n % size
    }

    fn has_nonzero_residue(n: Wrapping<u32>) -> bool {
        Self::residue(n) != Wrapping(0)
    }

    pub fn is_empty(&self) -> bool {
        Self::has_nonzero_residue(self.write_index() - self.read_index())
    }

    pub fn is_full(&self) -> bool {
        Self::has_nonzero_residue(self.write_index() - self.read_index() + Wrapping(1))
    }

    pub fn enqueue(&mut self, desc: Descriptor) -> Result<(), Error> {
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

    pub fn dequeue(&mut self) -> Result<Descriptor, Error> {
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

pub enum Error {
    RingIsFull,
    RingIsEmpty,
}
