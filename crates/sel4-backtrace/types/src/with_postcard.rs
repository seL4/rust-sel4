//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use postcard::ser_flavors::Flavor;
use serde::Serialize;

use crate::{Entry, Postamble, Preamble};

struct LameFlavor<F> {
    send_byte: F,
}

impl<F> LameFlavor<F> {
    fn new(send_byte: F) -> Self {
        Self { send_byte }
    }
}

impl<F: FnMut(u8) -> Result<(), E>, E> Flavor for &mut LameFlavor<F> {
    type Output = ();

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        (self.send_byte)(data).map_err(|_| postcard::Error::SerdeSerCustom)
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(())
    }
}

impl<T: Serialize> Preamble<T> {
    pub fn send<F: FnMut(u8) -> Result<(), E>, E>(&self, send_byte: F) -> postcard::Result<()> {
        postcard::serialize_with_flavor(self, &mut LameFlavor::new(send_byte))
    }
}

impl Entry {
    pub fn send<F: FnMut(u8) -> Result<(), E>, E>(&self, send_byte: F) -> postcard::Result<()> {
        let mut flavor = LameFlavor::new(send_byte);
        postcard::serialize_with_flavor(&true, &mut flavor)?;
        postcard::serialize_with_flavor(self, &mut flavor)?;
        Ok(())
    }
}

impl Postamble {
    pub fn send<F: FnMut(u8) -> Result<(), E>, E>(&self, send_byte: F) -> postcard::Result<()> {
        let mut flavor = LameFlavor::new(send_byte);
        postcard::serialize_with_flavor(&false, &mut flavor)?;
        postcard::serialize_with_flavor(self, &mut flavor)?;
        Ok(())
    }
}
