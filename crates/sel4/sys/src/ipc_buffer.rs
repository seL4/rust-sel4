//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use core::mem;
use core::ops::Range;
use core::slice;

use sel4_bitfield_ops::{get_bits, set_bits, set_bits_from_slice, PrimInt, UnsignedPrimInt};

use crate::{seL4_CPtr, seL4_IPCBuffer, seL4_Word};

impl seL4_IPCBuffer {
    pub(crate) fn get_mr(&self, i: usize) -> seL4_Word {
        self.msg[i]
    }

    pub(crate) fn set_mr(&mut self, i: usize, value: seL4_Word) {
        self.msg[i] = value;
    }

    pub(crate) fn get_mr_bits<T>(&self, range: Range<usize>) -> T
    where
        T: PrimInt,
        T::Unsigned: TryFrom<seL4_Word>,
    {
        T::cast_from_unsigned(get_bits(&self.msg, range))
    }

    pub(crate) fn set_mr_bits<T>(&mut self, range: Range<usize>, value: T)
    where
        T: PrimInt,
        T::Unsigned: TryInto<seL4_Word>,
    {
        set_bits(&mut self.msg, range, T::cast_to_unsigned(value))
    }

    pub(crate) fn set_mr_bits_from_slice<T>(&mut self, range: Range<usize>, value: &[T])
    where
        T: UnsignedPrimInt,
        usize: TryFrom<T>,
    {
        set_bits_from_slice(&mut self.msg, range, value, 0)
    }

    pub(crate) fn msg_bytes_mut(&mut self) -> &'static mut [u8] {
        let msg = &mut self.msg;
        unsafe {
            slice::from_raw_parts_mut(
                msg.as_mut_ptr().cast::<u8>(),
                msg.len() * mem::size_of::<seL4_Word>(),
            )
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_cap(&self, i: usize) -> seL4_CPtr {
        self.caps_or_badges[i]
    }

    pub(crate) fn set_cap(&mut self, i: usize, cptr: seL4_CPtr) {
        self.caps_or_badges[i] = cptr;
    }
}
