//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![no_std]

extern crate alloc;

use alloc::rc::Rc;

use addr2line::Context as AbstractContext;
use object::{Object, ObjectSection};

pub type Context = AbstractContext<gimli::EndianRcSlice<gimli::RunTimeEndian>>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
    ObjectError(object::Error),
    GimliError(gimli::Error),
}

impl From<object::Error> for Error {
    fn from(err: object::Error) -> Self {
        Self::ObjectError(err)
    }
}

impl From<gimli::Error> for Error {
    fn from(err: gimli::Error) -> Self {
        Self::GimliError(err)
    }
}

pub fn new_context<'data: 'file, 'file, O: Object<'data>>(
    file: &'file O,
) -> Result<Context, Error> {
    new_context_with_sup(file, None)
}

pub fn new_context_with_sup<'data: 'file, 'file, O: Object<'data>>(
    file: &'file O,
    sup_file: Option<&'file O>,
) -> Result<Context, Error> {
    let endian = match file.endianness() {
        object::Endianness::Little => gimli::RunTimeEndian::Little,
        object::Endianness::Big => gimli::RunTimeEndian::Big,
    };
    let mut dwarf = gimli::Dwarf::load(|id| load_section(id, file, endian))?;
    if let Some(sup_file) = sup_file {
        dwarf.load_sup(|id| load_section(id, sup_file, endian))?;
    }
    Ok(Context::from_dwarf(dwarf)?)
}

fn load_section<'data: 'file, 'file, O, Endian>(
    id: gimli::SectionId,
    file: &'file O,
    endian: Endian,
) -> Result<gimli::EndianRcSlice<Endian>, Error>
where
    O: Object<'data>,
    Endian: gimli::Endianity,
{
    let data = match file.section_by_name(id.name()) {
        Some(section) => section.uncompressed_data()?,
        None => Default::default(),
    };
    Ok(gimli::EndianRcSlice::new(Rc::from(data), endian))
}
