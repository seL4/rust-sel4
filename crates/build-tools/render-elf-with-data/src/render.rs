use anyhow::{anyhow, ensure, Result};
use object::{
    elf::{ProgramHeader64, PF_R, PF_W, PT_LOAD},
    read::elf::{ElfFile64, ProgramHeader as _},
    write::elf::{FileHeader, ProgramHeader, Writer},
    Endian, Endianness, Object, ObjectSegment, ObjectSymbol,
};

use super::*;

const TRIVIAL_ALIGN: usize = 1;

impl<'a> Input<'a> {
    pub fn render_with_data(&self, orig: &[u8]) -> Result<Vec<u8>> {
        let orig_obj: &ElfFile64 = &ElfFile64::parse(orig)?;
        let orig_endian = orig_obj.endian();
        let orig_image_end = usize::try_from(next_vaddr(orig_obj)?)?;

        let mut out_buf = vec![];
        let mut writer = Writer::new(orig_endian, orig_obj.is_64(), &mut out_buf);

        writer.reserve_file_header();
        writer.reserve_program_headers(
            (loadable_segments(orig_obj).count() + self.symbolic_injections.len()).try_into()?,
        );

        let revised_offsets = loadable_segments(orig_obj)
            .map(|phdr| {
                let filesz: usize = phdr.p_filesz.get(orig_endian).try_into().unwrap();
                writer.reserve(filesz, TRIVIAL_ALIGN)
            })
            .collect::<Vec<usize>>();

        let new_segments = {
            let mut next_vaddr = orig_image_end;
            self.symbolic_injections
                .iter()
                .map(|symbolic_injection| {
                    let filesz = symbolic_injection.filesz();
                    let offset = writer.reserve(filesz, TRIVIAL_ALIGN);
                    let vaddr = symbolic_injection.align_from(next_vaddr);
                    next_vaddr = vaddr + filesz;
                    Ok((symbolic_injection.locate(vaddr)?, offset))
                })
                .collect::<Result<Vec<(Injection, usize)>>>()?
        };

        writer.write_file_header({
            let hdr = orig_obj.raw_header();
            &FileHeader {
                os_abi: hdr.e_ident.os_abi,
                abi_version: hdr.e_ident.abi_version,
                e_type: hdr.e_type.get(orig_endian),
                e_machine: hdr.e_machine.get(orig_endian),
                e_entry: hdr.e_entry.get(orig_endian),
                e_flags: hdr.e_flags.get(orig_endian),
            }
        })?;

        writer.write_align_program_headers();

        for (phdr, revised_offset) in loadable_segments(orig_obj).zip(&revised_offsets) {
            writer.write_program_header(&ProgramHeader {
                p_type: phdr.p_type.get(orig_endian),
                p_flags: phdr.p_flags.get(orig_endian),
                p_offset: (*revised_offset).try_into()?,
                p_vaddr: phdr.p_vaddr.get(orig_endian),
                p_paddr: phdr.p_paddr.get(orig_endian),
                p_filesz: phdr.p_filesz.get(orig_endian),
                p_memsz: phdr.p_memsz.get(orig_endian),
                p_align: TRIVIAL_ALIGN.try_into().unwrap(),
            });
        }

        for (injection, offset) in &new_segments {
            let vaddr = injection.vaddr().try_into()?;
            writer.write_program_header(&ProgramHeader {
                p_type: PT_LOAD,
                p_flags: PF_R | PF_W,
                p_offset: (*offset).try_into()?,
                p_vaddr: vaddr,
                p_paddr: vaddr,
                p_filesz: injection.filesz().try_into()?,
                p_memsz: injection.memsz().try_into()?,
                p_align: TRIVIAL_ALIGN.try_into().unwrap(),
            });
        }

        for (phdr, revised_offset) in loadable_segments(orig_obj).zip(&revised_offsets) {
            writer.pad_until(*revised_offset);
            writer.write(phdr.data(orig_endian, orig).unwrap());
        }

        for (injection, offset) in &new_segments {
            writer.pad_until(*offset);
            writer.write(injection.content());
        }

        let mut recorded: Vec<(String, u64)> = vec![];
        {
            let image_start = first_vaddr(orig_obj)?;
            let image_end = new_segments
                .iter()
                .map(|(injection, _offset)| injection.vaddr() + injection.memsz())
                .max()
                .unwrap_or(orig_image_end)
                .try_into()?;
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

        let out_obj: &ElfFile64 = &ElfFile64::parse(out_buf.as_slice())?;
        let mut patches = vec![];
        for (name, value) in &recorded {
            let patch_offset =
                vaddr_to_offset(out_obj, get_symbol_vaddr(orig_obj, name)?)?.try_into()?;
            let value_bytes = out_obj.endian().write_u64_bytes(*value);
            patches.push((patch_offset, value_bytes))
        }
        for (patch_offset, value_bytes) in patches {
            out_buf[patch_offset..patch_offset + value_bytes.len()].copy_from_slice(&value_bytes);
        }

        Ok(out_buf)
    }
}

fn loadable_segments<'a>(
    obj: &'a ElfFile64,
) -> impl Iterator<Item = &'a ProgramHeader64<Endianness>> {
    obj.raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type.get(obj.endian()) == PT_LOAD)
}

fn get_symbol_vaddr(obj: &ElfFile64, name: &str) -> Result<u64> {
    for symbol in obj.symbols() {
        if symbol.name()? == name {
            ensure!(symbol.size() == 8);
            return Ok(symbol.address());
        }
    }
    Err(anyhow!("symbol '{}' not present", name))
}

fn vaddr_to_offset(obj: &ElfFile64, vaddr: u64) -> Result<u64> {
    for segment in obj.segments() {
        let start = segment.address();
        let end = start + segment.size();
        if (start..end).contains(&vaddr) {
            let offset_in_segment = vaddr - start;
            let (file_start, file_size) = segment.file_range();
            ensure!(offset_in_segment < file_size);
            return Ok(file_start + offset_in_segment);
        }
    }
    Err(anyhow!("vaddr '0x{:x}' not mapped", vaddr))
}

fn first_vaddr(obj: &ElfFile64) -> Result<u64> {
    loadable_segments(obj)
        .map(|phdr| phdr.p_vaddr.get(obj.endian()))
        .min()
        .ok_or(anyhow!("no segments"))
}

fn next_vaddr(obj: &ElfFile64) -> Result<u64> {
    loadable_segments(obj)
        .map(|phdr| phdr.p_vaddr.get(obj.endian()) + phdr.p_memsz.get(obj.endian()))
        .max()
        .ok_or(anyhow!("no segments"))
}
