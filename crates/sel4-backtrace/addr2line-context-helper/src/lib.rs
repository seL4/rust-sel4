//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::rc::Rc;

use addr2line::gimli;
use addr2line::object::{Object, ObjectSection};
use addr2line::Context as AbstractContext;

pub use addr2line::gimli::Error;

pub type Context = AbstractContext<gimli::EndianRcSlice<gimli::RunTimeEndian>>;

pub fn new_context<'data: 'file, 'file, O: Object<'data, 'file>>(
    file: &'file O,
) -> Result<Context, Error> {
    new_context_with_sup(file, None)
}

pub fn new_context_with_sup<'data: 'file, 'file, O: Object<'data, 'file>>(
    file: &'file O,
    sup_file: Option<&'file O>,
) -> Result<Context, Error> {
    let endian = if file.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    fn load_section<'data: 'file, 'file, O, Endian>(
        id: gimli::SectionId,
        file: &'file O,
        endian: Endian,
    ) -> Result<gimli::EndianRcSlice<Endian>, Error>
    where
        O: Object<'data, 'file>,
        Endian: gimli::Endianity,
    {
        let data = file
            .section_by_name(id.name())
            .and_then(|section| section.uncompressed_data().ok())
            .unwrap_or(Cow::Borrowed(&[]));
        Ok(gimli::EndianRcSlice::new(Rc::from(&*data), endian))
    }

    let mut dwarf = gimli::Dwarf::load(|id| load_section(id, file, endian))?;
    if let Some(sup_file) = sup_file {
        dwarf.load_sup(|id| load_section(id, sup_file, endian))?;
    }
    Context::from_dwarf(dwarf)
}
