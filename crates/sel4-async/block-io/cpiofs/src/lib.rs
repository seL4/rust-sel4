#![no_std]
#![feature(async_fn_in_trait)]
#![feature(int_roundings)]
#![feature(slice_as_chunks)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use core::mem;

use hex::FromHex;
use zerocopy::{AsBytes, FromBytes, FromZeroes};

use sel4_async_block_io::{access::ReadOnly, ByteIO};

const CPIO_ALIGN: usize = 4;

const END_OF_ARCHIVE: &str = "TRAILER!!!";

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromBytes, FromZeroes)]
struct HexEncodedU32 {
    encoded: [u8; 8],
}

impl HexEncodedU32 {
    fn get(&self) -> u32 {
        u32::from_be_bytes(FromHex::from_hex(self.encoded).unwrap())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, FromBytes, FromZeroes)]
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

    async fn read_entry<T: ByteIO<ReadOnly>>(&self, io: &T) -> Result<Entry, T::Error> {
        let mut header = Header::new_zeroed();
        io.read(self.offset().try_into().unwrap(), header.as_bytes_mut())
            .await?;
        header.check_magic();
        Ok(Entry {
            header,
            location: *self,
        })
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

    async fn read_name<T: ByteIO<ReadOnly>>(&self, io: &T) -> Result<String, T::Error> {
        let mut buf = vec![0; self.header().name_size()];
        io.read(self.name_offset().try_into().unwrap(), &mut buf)
            .await?;
        assert_eq!(buf.pop().unwrap(), 0);
        Ok(String::from_utf8(buf).unwrap())
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

impl<T: ByteIO<ReadOnly>> Index<T> {
    pub async fn create(io: T) -> Result<Self, T::Error> {
        let mut entries = BTreeMap::new();
        let mut location = EntryLocation::first();
        loop {
            let entry = location.read_entry(&io).await?;
            let path = entry.read_name(&io).await?;
            if path == END_OF_ARCHIVE {
                break;
            }
            location = entry.next_entry_location();
            entries.insert(path, entry.location);
        }
        Ok(Self { entries, io })
    }

    pub fn lookup(&self, path: &str) -> Option<&EntryLocation> {
        self.entries.get(path)
    }

    pub fn entries(&self) -> &BTreeMap<String, EntryLocation> {
        &self.entries
    }

    pub async fn read_entry(&self, location: &EntryLocation) -> Result<Entry, T::Error> {
        location.read_entry(&self.io).await
    }

    pub async fn read_data(
        &self,
        entry: &Entry,
        offset_into_data: usize,
        buf: &mut [u8],
    ) -> Result<(), T::Error> {
        let offset = entry.data_offset() + offset_into_data;
        self.io.read(offset.try_into().unwrap(), buf).await
    }
}
