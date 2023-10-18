use core::mem;
use core::ops::Range;

use gpt_disk_types::{GptHeader, MasterBootRecord, MbrPartitionRecord};
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{access::ReadOnly, read_bytes, BlockIO, Partition};

pub struct Disk<T> {
    io: T,
}

#[derive(Debug)]
pub enum DiskError<E> {
    IOError(E),
    MbrInvalidSignature,
}

impl<E> From<E> for DiskError<E> {
    fn from(io_error: E) -> Self {
        Self::IOError(io_error)
    }
}

pub struct Mbr {
    inner: MasterBootRecord,
}

impl Mbr {
    fn new<E>(inner: MasterBootRecord) -> Result<Self, DiskError<E>> {
        if inner.signature != [0x55, 0xaa] {
            return Err(DiskError::MbrInvalidSignature);
        }
        Ok(Self { inner })
    }

    pub fn disk_signature(&self) -> [u8; 4] {
        self.inner.unique_mbr_disk_signature
    }

    pub fn partition(&self, i: usize) -> Option<MbrPartitionEntry> {
        self.inner
            .partitions
            .get(i)
            .copied()
            .map(MbrPartitionEntry::new)
    }
}

pub struct MbrPartitionEntry {
    inner: MbrPartitionRecord,
}

impl MbrPartitionEntry {
    fn new(inner: MbrPartitionRecord) -> Self {
        Self { inner }
    }

    pub fn partition_id(&self) -> PartitionId {
        self.inner.os_indicator.into()
    }

    fn lba_range(&self) -> Range<u64> {
        let start = self.inner.starting_lba.to_u32().into();
        let size = self.inner.size_in_lba.to_u32().into();
        start..start.checked_add(size).unwrap()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum KnownPartitionId {
    Free = 0x00,
    Fat32 = 0x0c,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum PartitionId {
    Known(KnownPartitionId),
    Unknown(u8),
}

impl From<u8> for PartitionId {
    fn from(val: u8) -> Self {
        KnownPartitionId::try_from(val)
            .map(Self::Known)
            .unwrap_or_else(|_| Self::Unknown(val))
    }
}

impl<T: BlockIO<ReadOnly>> Disk<T> {
    pub fn new(io: T) -> Self {
        Self { io }
    }

    fn io(&self) -> &T {
        &self.io
    }

    pub async fn read_mbr(&self) -> Result<Mbr, DiskError<T::Error>> {
        let mut buf = [0; mem::size_of::<MasterBootRecord>()];
        read_bytes(self.io(), 0, &mut buf[..]).await?;
        Mbr::new(*bytemuck::from_bytes(&buf[..]))
    }

    pub async fn read_gpt_header(&self) -> Result<GptHeader, T::Error> {
        let mut buf = [0; mem::size_of::<GptHeader>()];
        read_bytes(self.io(), 0, &mut buf[..]).await?;
        Ok(*bytemuck::from_bytes(&buf[..]))
    }
}

impl<T: BlockIO<ReadOnly>> Disk<T> {
    pub fn partition_using_mbr(self, entry: &MbrPartitionEntry) -> Partition<T> {
        Partition::new(self.io, entry.lba_range())
    }
}
