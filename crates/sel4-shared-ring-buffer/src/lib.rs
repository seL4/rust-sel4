#![no_std]

use core::num::Wrapping;
use core::sync::atomic::{fence, Ordering};

use zerocopy::{AsBytes, FromBytes};

use sel4_externally_shared::ExternallyShared;

pub struct RingBuffers<F> {
    free: RingBuffer,
    used: RingBuffer,
    notify: F,
}

impl<F> RingBuffers<F> {
    pub fn new(free: RingBuffer, used: RingBuffer, notify: F, initialize: bool) -> Self {
        let mut this = Self { free, used, notify };
        if initialize {
            this.free_mut().initialize();
            this.used_mut().initialize();
        }
        this
    }

    pub fn free(&self) -> &RingBuffer {
        &self.free
    }

    pub fn used(&self) -> &RingBuffer {
        &self.used
    }

    pub fn free_mut(&mut self) -> &mut RingBuffer {
        &mut self.free
    }

    pub fn used_mut(&mut self) -> &mut RingBuffer {
        &mut self.used
    }
}

impl<F: Fn() -> R, R> RingBuffers<F> {
    pub fn notify(&self) -> R {
        (self.notify)()
    }
}

impl<F: FnMut() -> R, R> RingBuffers<F> {
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

pub struct RingBuffer {
    inner: ExternallyShared<&'static mut RawRingBuffer>,
}

impl RingBuffer {
    pub const SIZE: usize = 512;

    pub unsafe fn new(inner: ExternallyShared<&'static mut RawRingBuffer>) -> Self {
        Self { inner }
    }

    pub unsafe fn from_ptr(ptr: *mut RawRingBuffer) -> Self {
        Self::new(ExternallyShared::new(ptr.as_mut().unwrap()))
    }

    fn write_index(&self) -> Wrapping<u32> {
        Wrapping(self.inner.map(|r| &r.write_index).read())
    }

    fn read_index(&self) -> Wrapping<u32> {
        Wrapping(self.inner.map(|r| &r.read_index).read())
    }

    fn set_write_index(&mut self, index: Wrapping<u32>) {
        self.inner.map_mut(|r| &mut r.write_index).write(index.0)
    }

    fn set_read_index(&mut self, index: Wrapping<u32>) {
        self.inner.map_mut(|r| &mut r.read_index).write(index.0)
    }

    fn initialize(&mut self) {
        self.set_write_index(Wrapping(0));
        self.set_read_index(Wrapping(0));
    }

    fn descriptor(&mut self, index: Wrapping<u32>) -> ExternallyShared<&mut Descriptor> {
        let linear_index = usize::try_from(Self::residue(index).0).unwrap();
        self.inner.map_mut(|r| &mut r.descriptors[linear_index])
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
        let index = self.write_index();
        self.descriptor(index).write(desc);
        release();
        self.set_write_index(index + Wrapping(1));
        Ok(())
    }

    pub fn dequeue(&mut self) -> Result<Descriptor, Error> {
        if self.is_empty() {
            return Err(Error::RingIsEmpty);
        }
        let index = self.read_index();
        let desc = self.descriptor(index).read();
        release();
        self.set_read_index(index + Wrapping(1));
        Ok(desc)
    }
}

fn release() {
    fence(Ordering::Release);
}

pub enum Error {
    RingIsFull,
    RingIsEmpty,
}
