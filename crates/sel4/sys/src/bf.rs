use core::fmt;

use sel4_bitfield_ops::Bitfield;

pub(crate) type SeL4Bitfield<T, const N: usize> = Bitfield<[T; N], T>;

include!(concat!(env!("OUT_DIR"), "/types.rs"));
include!(concat!(env!("OUT_DIR"), "/shared_types.rs"));
