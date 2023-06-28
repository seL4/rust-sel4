#![no_std]
#![feature(async_fn_in_trait)]
#![feature(int_roundings)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use core::cell::RefCell;
use core::marker::PhantomData;
use core::mem;
use core::num::NonZeroUsize;

use hex::FromHex;
use lru::LruCache;
use zerocopy::{AsBytes, FromBytes};

const CPIO_ALIGN: usize = 4;

const END_OF_ARCHIVE: &str = "TRAILER!!!";

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromBytes)]
struct HexEncodedU32 {
    encoded: [u8; 8],
}

impl HexEncodedU32 {
    fn get(&self) -> u32 {
        u32::from_be_bytes(FromHex::from_hex(&self.encoded).unwrap())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromBytes)]
struct Header {
    c_magic: [u8; 6],
    c_ino: HexEncodedU32,
    c_mode: HexEncodedU32,
    c_uid: HexEncodedU32,
    c_gid: HexEncodedU32,
    c_nlink: HexEncodedU32,
    c_mtime: HexEncodedU32,
    c_filesize: HexEncodedU32,
    c_maj: HexEncodedU32,
    c_min: HexEncodedU32,
    c_rmaj: HexEncodedU32,
    c_rmin: HexEncodedU32,
    c_namesize: HexEncodedU32,
    c_chksum: HexEncodedU32,
}

impl Header {
    fn check_magic(&self) {
        let ok = &self.c_magic == b"070701" || &self.c_magic == b"070702";
        assert!(ok);
    }

    fn file_size(&self) -> usize {
        self.c_filesize.get().try_into().unwrap()
    }

    fn name_size(&self) -> usize {
        self.c_namesize.get().try_into().unwrap()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct EntryLocation {
    offset: usize,
}

impl EntryLocation {
    fn first() -> Self {
        Self { offset: 0 }
    }

    fn offset(&self) -> usize {
        self.offset
    }

    async fn read_entry<T: IO>(&self, io: &T) -> Entry {
        let mut header = Header::new_zeroed();
        io.read(self.offset(), header.as_bytes_mut()).await;
        header.check_magic();
        Entry {
            header,
            location: self.clone(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Entry {
    header: Header,
    location: EntryLocation,
}

impl Entry {
    pub fn data_size(&self) -> usize {
        self.header().file_size()
    }

    pub fn ty(&self) -> EntryType {
        match self.header().c_mode.get() & 0o0170000 {
            0o0120000 => EntryType::SymbolicLink,
            0o0100000 => EntryType::RegularFile,
            0o0040000 => EntryType::Directory,
            _ => panic!(),
        }
    }

    pub fn location(&self) -> &EntryLocation {
        &self.location
    }

    fn header(&self) -> &Header {
        &self.header
    }

    fn name_offset(&self) -> usize {
        self.location().offset() + mem::size_of::<Header>()
    }

    fn data_offset(&self) -> usize {
        (self.name_offset() + self.header().name_size()).next_multiple_of(CPIO_ALIGN)
    }

    fn next_entry_location(&self) -> EntryLocation {
        EntryLocation {
            offset: (self.data_offset() + self.header().file_size()).next_multiple_of(CPIO_ALIGN),
        }
    }

    async fn read_name<T: IO>(&self, io: &T) -> String {
        let mut buf = vec![0; self.header().name_size()];
        io.read(self.name_offset(), &mut buf).await;
        assert_eq!(buf.pop().unwrap(), 0);
        String::from_utf8(buf).unwrap()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntryType {
    RegularFile,
    Directory,
    SymbolicLink,
}

pub struct Index<T> {
    entries: BTreeMap<String, EntryLocation>,
    io: T,
}

impl<T: IO> Index<T> {
    pub async fn create(io: T) -> Self {
        let mut entries = BTreeMap::new();
        let mut location = EntryLocation::first();
        loop {
            let entry = location.read_entry(&io).await;
            let path = entry.read_name(&io).await;
            if path == END_OF_ARCHIVE {
                break;
            }
            location = entry.next_entry_location();
            entries.insert(path, entry.location.clone());
        }
        Self { entries, io }
    }

    pub fn lookup(&self, path: &str) -> Option<&EntryLocation> {
        self.entries.get(path)
    }

    pub fn entries(&self) -> &BTreeMap<String, EntryLocation> {
        &self.entries
    }

    pub async fn read_entry(&self, location: &EntryLocation) -> Entry {
        location.read_entry(&self.io).await
    }

    pub async fn read_data(&self, entry: &Entry, offset_into_data: usize, buf: &mut [u8]) {
        let offset = entry.data_offset() + offset_into_data;
        self.io.read(offset, buf).await;
    }
}

pub trait IO {
    async fn read(&self, offset: usize, buf: &mut [u8]);
}

// NOTE: type gymnastics due to current limitations of generic_const_exprs

pub type BlockId = usize;

pub trait BlockIO<const BLOCK_SIZE: usize> {
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]);
}

#[derive(Clone, Debug)]
pub struct BlockIOAdapter<T, const BLOCK_SIZE: usize> {
    inner: T,
    _phantom: PhantomData<[(); BLOCK_SIZE]>,
}

impl<T, const BLOCK_SIZE: usize> BlockIOAdapter<T, BLOCK_SIZE> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<const BLOCK_SIZE: usize, T: BlockIO<BLOCK_SIZE>> IO for BlockIOAdapter<T, BLOCK_SIZE> {
    async fn read(&self, offset: usize, buf: &mut [u8]) {
        let mut block_buf = [0; BLOCK_SIZE];
        let start_offset = offset;
        let end_offset = offset + buf.len();
        let start_block_id = start_offset / BLOCK_SIZE;
        let end_block_id = end_offset.next_multiple_of(BLOCK_SIZE) / BLOCK_SIZE;
        for block_id in start_block_id..end_block_id {
            self.inner.read_block(block_id, &mut block_buf).await;
            let this_start_offset = start_offset.max(block_id * BLOCK_SIZE);
            let this_end_offset = end_offset.min((block_id + 1) * BLOCK_SIZE);
            let this_len = this_end_offset - this_start_offset;
            buf[this_start_offset - start_offset..this_end_offset - start_offset]
                .copy_from_slice(&block_buf[this_start_offset % BLOCK_SIZE..][..this_len]);
        }
    }
}

#[derive(Debug)]
pub struct CachedBlockIO<T, const BLOCK_SIZE: usize> {
    inner: T,
    lru: RefCell<LruCache<BlockId, [u8; BLOCK_SIZE]>>,
}

impl<T, const BLOCK_SIZE: usize> CachedBlockIO<T, BLOCK_SIZE> {
    pub fn new(inner: T, cache_size_in_blocks: usize) -> Self {
        Self {
            inner,
            lru: RefCell::new(LruCache::new(
                NonZeroUsize::new(cache_size_in_blocks).unwrap(),
            )),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: BlockIO<BLOCK_SIZE>, const BLOCK_SIZE: usize> BlockIO<BLOCK_SIZE>
    for CachedBlockIO<T, BLOCK_SIZE>
{
    async fn read_block(&self, block_id: usize, buf: &mut [u8; BLOCK_SIZE]) {
        // odd control flow to avoid holding core::cell::RefMut across await
        if let Some(block) = self.lru.borrow_mut().get(&block_id) {
            *buf = block.clone();
            return;
        }
        self.inner().read_block(block_id, buf).await;
        let _ = self.lru.borrow_mut().put(block_id, buf.clone());
    }
}
