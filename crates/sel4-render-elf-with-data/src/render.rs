//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::mem;

use anyhow::{anyhow, ensure, Result};
use num::{CheckedAdd, NumCast, ToPrimitive};
use object::{
    elf::{PF_R, PF_W, PT_LOAD},
    read::elf::{ElfFile, FileHeader, ProgramHeader as _},
    write::elf::{ProgramHeader, Writer},
    Object, ObjectSegment, ObjectSymbol,
};

use crate::{FileHeaderExt, Injection, Input};

impl<'a, T: FileHeaderExt> Input<'a, T> {
    pub fn render_with_data(&self, orig: &[u8]) -> Result<Vec<u8>> {
        let orig_obj: &ElfFile<T> = &ElfFile::parse(orig)?;
        let orig_endian = orig_obj.endian();
        let orig_image_end = next_vaddr(orig_obj)?;

        let mut out_buf = vec![];
        let mut writer = Writer::new(orig_endian, orig_obj.is_64(), &mut out_buf);

        writer.reserve_file_header();
        writer.reserve_program_headers(
            (loadable_segments(orig_obj).count() + self.symbolic_injections.len()).try_into()?,
        );

        let revised_offsets = loadable_segments(orig_obj)
            .map(|phdr| {
                let filesz = phdr.p_filesz(orig_endian).to_usize().unwrap();
                let offset = writer.reserve(filesz, 1);
                Ok(offset)
            })
            .collect::<Result<Vec<usize>>>()?;

        let new_segments = {
            let mut next_vaddr = orig_image_end;
            self.symbolic_injections
                .iter()
                .map(|symbolic_injection| {
                    let filesz = symbolic_injection.filesz();
                    let offset = writer.reserve(filesz.to_usize().unwrap(), 1);
                    let vaddr = symbolic_injection.align_from(next_vaddr);
                    next_vaddr = vaddr.checked_add(&filesz).unwrap();
                    Ok((symbolic_injection.locate(vaddr)?, offset))
                })
                .collect::<Result<Vec<(Injection<_>, usize)>>>()?
        };

        writer.write_file_header({
            let hdr = orig_obj.raw_header();
            &object::write::elf::FileHeader {
                os_abi: hdr.e_ident().os_abi,
                abi_version: hdr.e_ident().abi_version,
                e_type: hdr.e_type(orig_endian),
                e_machine: hdr.e_machine(orig_endian),
                e_entry: hdr.e_entry(orig_endian).into(),
                e_flags: hdr.e_flags(orig_endian),
            }
        })?;

        writer.write_align_program_headers();

        for (phdr, revised_offset) in loadable_segments(orig_obj).zip(&revised_offsets) {
            writer.write_program_header(&ProgramHeader {
                p_type: phdr.p_type(orig_endian),
                p_flags: phdr.p_flags(orig_endian),
                p_offset: (*revised_offset).try_into()?,
                p_vaddr: phdr.p_vaddr(orig_endian).into(),
                p_paddr: phdr.p_paddr(orig_endian).into(),
                p_filesz: phdr.p_filesz(orig_endian).into(),
                p_memsz: phdr.p_memsz(orig_endian).into(),
                p_align: 1,
            });
        }

        for (injection, offset) in &new_segments {
            let vaddr = injection.vaddr();
            writer.write_program_header(&ProgramHeader {
                p_type: PT_LOAD,
                p_flags: PF_R | PF_W,
                p_offset: (*offset).try_into()?,
                p_vaddr: vaddr.into(),
                p_paddr: vaddr.into(),
                p_filesz: injection.filesz().into(),
                p_memsz: injection.memsz().into(),
                p_align: 1,
            });
        }

        for (phdr, revised_offset) in loadable_segments(orig_obj).zip(&revised_offsets) {
            writer.pad_until(*revised_offset);
            writer.write(phdr.data(orig_endian, orig).ok().unwrap());
        }

        for (injection, offset) in &new_segments {
            writer.pad_until(*offset);
            writer.write(injection.content());
        }

        let mut recorded: Vec<(String, T::Word)> = vec![];
        {
            let image_start = first_vaddr(orig_obj)?;
            let image_end = new_segments
                .iter()
                .map(|(injection, _offset)| injection.vaddr() + injection.memsz())
                .max()
                .unwrap_or(orig_image_end);
            for name in &self.image_start_patches {
                recorded.push((name.clone(), image_start))
            }
            for name in &self.image_end_patches {
                recorded.push((name.clone(), image_end))
            }
        }
        for (injection, _offset) in &new_segments {
            for (name, value) in injection.patches() {
                recorded.push((name.clone(), *value))
            }
        }
        for (name, value) in &self.concrete_patches {
            recorded.push((name.clone(), *value))
        }

        let out_obj: &ElfFile<T> = &ElfFile::parse(out_buf.as_slice())?;
        let mut patches = vec![];
        for (name, value) in &recorded {
            let patch_offset =
                vaddr_to_offset(out_obj, get_symbol_vaddr(orig_obj, name)?)?.try_into()?;
            let value_bytes = T::write_word_bytes(out_obj.endian(), *value);
            patches.push((patch_offset, value_bytes))
        }
        for (patch_offset, value_bytes) in patches {
            out_buf[patch_offset..patch_offset + value_bytes.len()].copy_from_slice(&value_bytes);
        }

        Ok(out_buf)
    }
}

fn loadable_segments<'a, T: FileHeaderExt>(
    obj: &'a ElfFile<T>,
) -> impl Iterator<Item = &'a <T as FileHeader>::ProgramHeader> {
    obj.raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type(obj.endian()) == PT_LOAD)
}

fn get_symbol_vaddr<T: FileHeaderExt>(obj: &ElfFile<T>, name: &str) -> Result<T::Word> {
    for symbol in obj.symbols() {
        if symbol.name()? == name {
            ensure!(usize::try_from(symbol.size()).unwrap() == mem::size_of::<T::Word>());
            return Ok(NumCast::from(symbol.address()).unwrap());
        }
    }
    Err(anyhow!("symbol '{}' not present", name))
}

fn vaddr_to_offset<T: FileHeaderExt>(obj: &ElfFile<T>, vaddr: T::Word) -> Result<u64>
where
{
    for segment in obj.segments() {
        let start = segment.address();
        let end = start + segment.size();
        if (start..end).contains(&vaddr.into()) {
            let offset_in_segment = vaddr.into() - start;
            let (file_start, file_size) = segment.file_range();
            ensure!(offset_in_segment < file_size);
            return Ok(file_start + offset_in_segment);
        }
    }
    Err(anyhow!(
        "vaddr '0x{:x}' not mapped",
        <T::Word as Into<u64>>::into(vaddr)
    ))
}

fn first_vaddr<T: FileHeaderExt>(obj: &ElfFile<T>) -> Result<T::Word>
where
{
    loadable_segments(obj)
        .map(|phdr| phdr.p_vaddr(obj.endian()))
        .min()
        .ok_or(anyhow!("no segments"))
}

fn next_vaddr<T: FileHeaderExt>(obj: &ElfFile<T>) -> Result<T::Word>
where
{
    loadable_segments(obj)
        .map(|phdr| phdr.p_vaddr(obj.endian()) + phdr.p_memsz(obj.endian()))
        .max()
        .ok_or(anyhow!("no segments"))
}
