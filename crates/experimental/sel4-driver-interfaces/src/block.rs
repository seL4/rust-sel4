//
// Copyright 2024, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::cell::RefCell;
use core::fmt;
use core::ops::Deref;

use lock_api::{Mutex, RawMutex};
use serde::{Deserialize, Serialize};

use crate::{WrappedMutex, WrappedRefCell, WrappedRefCellError};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct MacAddress(pub [u8; 6]);

pub trait GetBlockDeviceLayout {
    type Error: fmt::Debug;

    fn get_block_size(&mut self) -> Result<usize, Self::Error>;

    fn get_num_blocks(&mut self) -> Result<u64, Self::Error>;
}

impl<T: Deref<Target = RefCell<U>>, U: GetBlockDeviceLayout> GetBlockDeviceLayout
    for &WrappedRefCell<T>
{
    type Error = WrappedRefCellError<U::Error>;

    fn get_block_size(&mut self) -> Result<usize, Self::Error> {
        self.with_mut(|this| this.get_block_size())
    }

    fn get_num_blocks(&mut self) -> Result<u64, Self::Error> {
        self.with_mut(|this| this.get_num_blocks())
    }
}

impl<R: RawMutex, T: Deref<Target = Mutex<R, U>>, U: GetBlockDeviceLayout> GetBlockDeviceLayout
    for &WrappedMutex<T>
{
    type Error = U::Error;

    fn get_block_size(&mut self) -> Result<usize, Self::Error> {
        self.with_mut(|this| this.get_block_size())
    }

    fn get_num_blocks(&mut self) -> Result<u64, Self::Error> {
        self.with_mut(|this| this.get_num_blocks())
    }
}
