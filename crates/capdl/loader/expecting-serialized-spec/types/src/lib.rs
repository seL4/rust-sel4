#![no_std]

extern crate alloc;

use alloc::string::String;

use capdl_types::*;

pub type SerializedSpec<'a> = Spec<'a, Option<String>, FillEntryContentDeflatedBytesVia>;
