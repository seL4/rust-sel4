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

pub trait GetNetDeviceMeta {
    type Error: fmt::Debug;

    fn get_mac_address(&mut self) -> Result<MacAddress, Self::Error>;
}

impl<T: Deref<Target = RefCell<U>>, U: GetNetDeviceMeta> GetNetDeviceMeta for &WrappedRefCell<T> {
    type Error = WrappedRefCellError<U::Error>;

    fn get_mac_address(&mut self) -> Result<MacAddress, Self::Error> {
        self.with_mut(|this| this.get_mac_address())
    }
}

impl<R: RawMutex, T: Deref<Target = Mutex<R, U>>, U: GetNetDeviceMeta> GetNetDeviceMeta
    for &WrappedMutex<T>
{
    type Error = U::Error;

    fn get_mac_address(&mut self) -> Result<MacAddress, Self::Error> {
        self.with_mut(|this| this.get_mac_address())
    }
}
