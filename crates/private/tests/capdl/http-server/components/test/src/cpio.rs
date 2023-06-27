#![allow(dead_code)]

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use core::mem;

use hex::FromHex;
use zerocopy::{AsBytes, FromBytes};

const CPIO_ALIGN: usize = 4;

const END_OF_ARCHIVE: &str = "TRAILER!!!";

#[repr(C)]
#[derive(Debug, Clone, AsBytes, FromBytes)]
struct HexEncodedU32 {
    encoded: [u8; 8],
}

impl HexEncodedU32 {
    fn get(&self) -> u32 {
        u32::from_be_bytes(FromHex::from_hex(&self.encoded).unwrap())
    }
}

#[repr(C)]
#[derive(Debug, Clone, AsBytes, FromBytes)]
pub struct Header {
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

#[derive(Debug, Clone)]
pub struct Entry {
    header: Header,
    offset: usize,
}

impl Entry {
    pub fn data_size(&self) -> usize {
        self.header().file_size()
    }

    fn header(&self) -> &Header {
        &self.header
    }

    fn name_offset(&self) -> usize {
        self.offset + mem::size_of::<Header>()
    }

    fn data_offset(&self) -> usize {
        (self.name_offset() + self.header().name_size()).next_multiple_of(CPIO_ALIGN)
    }

    fn next_header_offset(&self) -> usize {
        (self.data_offset() + self.header().file_size()).next_multiple_of(CPIO_ALIGN)
    }

    async fn read<T: IO>(offset: usize, io: &T) -> Self {
        let mut header = Header::new_zeroed();
        io.read(offset, header.as_bytes_mut()).await;
        header.check_magic();
        Self { header, offset }
    }

    async fn read_name<T: IO>(&self, io: &T) -> String {
        let mut buf = vec![0; self.header().name_size()];
        io.read(self.name_offset(), &mut buf).await;
        assert_eq!(buf.pop().unwrap(), 0);
        String::from_utf8(buf).unwrap()
    }
}

pub trait IO {
    async fn read(&self, offset: usize, buffer: &mut [u8]);
}

pub struct CpioIndex<T> {
    entries: BTreeMap<String, Entry>,
    io: T,
}

impl<T: IO> CpioIndex<T> {
    pub async fn create(io: T) -> Self {
        let mut entries = BTreeMap::new();
        let mut offset = 0;
        loop {
            let entry = Entry::read(offset, &io).await;
            let path = entry.read_name(&io).await;
            if path == END_OF_ARCHIVE {
                break;
            }
            offset = entry.next_header_offset();
            entries.insert(path, entry);
        }
        Self { entries, io }
    }

    pub fn lookup(&self, path: &str) -> Option<&Entry> {
        self.entries.get(path)
    }

    pub fn entries(&self) -> &BTreeMap<String, Entry> {
        &self.entries
    }

    pub async fn read(&self, entry: &Entry, offset_into_data: usize, buffer: &mut [u8]) {
        let offset = entry.data_offset() + offset_into_data;
        self.io.read(offset, buffer).await;
    }
}
