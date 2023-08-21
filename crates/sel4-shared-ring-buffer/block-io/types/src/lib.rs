#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use zerocopy::{AsBytes, FromBytes};

use sel4_shared_ring_buffer::Descriptor;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, AsBytes, FromBytes)]
#[repr(C)]
pub struct BlockIORequest {
    status: i32,
    ty: u32,
    block_id: usize,
    buf: Descriptor,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum BlockIORequestType {
    Read = 0,
    Write = 1,
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum BlockIORequestStatus {
    Pending = -1,
    Ok = 0,
    IOError = 1,
}

impl BlockIORequest {
    pub fn new(
        status: BlockIORequestStatus,
        ty: BlockIORequestType,
        block_id: usize,
        buf: Descriptor,
    ) -> Self {
        Self {
            status: status.into(),
            ty: ty.into(),
            block_id,
            buf,
        }
    }

    pub fn status(
        &self,
    ) -> Result<BlockIORequestStatus, TryFromPrimitiveError<BlockIORequestStatus>> {
        self.status.try_into()
    }

    pub fn set_status(&mut self, status: BlockIORequestStatus) {
        self.status = status.into();
    }

    pub fn ty(&self) -> Result<BlockIORequestType, TryFromPrimitiveError<BlockIORequestType>> {
        self.ty.try_into()
    }

    pub fn block_id(&self) -> usize {
        self.block_id
    }

    pub fn buf(&self) -> &Descriptor {
        &self.buf
    }
}
