//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};
use zerocopy::{FromBytes, Immutable, IntoBytes};

use sel4_shared_ring_buffer::Descriptor;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, FromBytes, IntoBytes, Immutable)]
#[repr(C)]
pub struct BlockIORequest {
    status: i32,
    ty: u32,
    start_block_idx: u64,
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
        start_block_idx: u64,
        buf: Descriptor,
    ) -> Self {
        Self {
            status: status.into(),
            ty: ty.into(),
            start_block_idx,
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

    pub fn set_ty(&mut self, ty: BlockIORequestType) {
        self.ty = ty.into();
    }

    pub fn start_block_idx(&self) -> u64 {
        self.start_block_idx
    }

    pub fn set_start_block_idx(&mut self, start_block_idx: u64) {
        self.start_block_idx = start_block_idx;
    }

    pub fn buf(&self) -> &Descriptor {
        &self.buf
    }

    pub fn buf_mut(&mut self) -> &mut Descriptor {
        &mut self.buf
    }
}
