use std::fs;

use anyhow::{anyhow, ensure, Result};
use object::{
    elf::{ProgramHeader64, PF_R, PF_W, PT_LOAD},
    read::elf::{ElfFile64, ProgramHeader as _},
    write::elf::{FileHeader, ProgramHeader, Writer},
    Endian, Endianness, Object, ObjectSegment, ObjectSymbol,
};

mod args;
mod injection;
mod utils;

use args::*;
use injection::*;
use utils::*;

pub use args::BoundsSymbolArgs;
pub use injection::{SymbolicInjection, DEFAULT_ALIGN};

pub fn main() -> Result<()> {
    let args = Args::parse()?;
    if args.verbose {
        eprintln!("{args:#?}");
    }
    let injections = args
        .injections
        .iter()
        .map(SymbolicInjection::from_arg)
        .collect::<Result<Vec<SymbolicInjection>>>()?;
    let in_file = fs::read(args.in_file_path)?;
    let out_buf = inject(in_file.as_slice(), injections, &args.image_bounds_symbols)?;
    fs::write(args.out_file_path, out_buf)?;
    Ok(())
}

struct InjectionMeta {
    offset: usize,
    p_align: usize, // TODO
}

pub fn inject(
    orig: &[u8],
    symbolic_injections: Vec<SymbolicInjection>,
    image_bounds_symbols: &BoundsSymbolArgs,
) -> Result<Vec<u8>> {
    let orig_obj: &ElfFile64 = &ElfFile64::parse(orig)?;
    let orig_endian = orig_obj.endian();
    let orig_image_end = next_vaddr(orig_obj)?.try_into()?;

    let mut out_buf = vec![];
    let mut writer = Writer::new(orig_endian, orig_obj.is_64(), &mut out_buf);

    writer.reserve_file_header();
    writer.reserve_program_headers(
        (loadable_segments(orig_obj).count() + symbolic_injections.len()).try_into()?,
    );

    let revised_offsets = loadable_segments(orig_obj)
        .map(|phdr| {
            let offset: usize = phdr.p_offset.get(orig_endian).try_into().unwrap();
            let filesz: usize = phdr.p_filesz.get(orig_endian).try_into().unwrap();
            let align = phdr.p_align.get(orig_endian).try_into().unwrap();
            let offset_within_alignment = offset % align;
            offset_within_alignment + writer.reserve(offset_within_alignment + filesz, align)
        })
        .collect::<Vec<usize>>();

    let new_segments = {
        let mut next_vaddr = orig_image_end;
        symbolic_injections
            .into_iter()
            .map(|symbolic_injection| {
                let size = symbolic_injection.size();
                let offset = writer.reserve(size, symbolic_injection.align());
                let vaddr = align_up(next_vaddr, symbolic_injection.align());
                let p_align = symbolic_injection.align(); // TODO
                next_vaddr = vaddr + size;
                Ok((
                    symbolic_injection.locate(vaddr)?,
                    InjectionMeta { offset, p_align },
                ))
            })
            .collect::<Result<Vec<(Injection, InjectionMeta)>>>()?
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
            p_align: phdr.p_align.get(orig_endian),
        });
    }

    for (injection, meta) in &new_segments {
        let offset = meta.offset.try_into()?;
        let vaddr = injection.vaddr().try_into()?;
        let size = injection.size().try_into()?;
        writer.write_program_header(&ProgramHeader {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_W,
            p_offset: offset,
            p_vaddr: vaddr,
            p_paddr: vaddr,
            p_filesz: size,
            p_memsz: size,
            p_align: meta.p_align.try_into()?,
        });
    }

    for (phdr, revised_offset) in loadable_segments(orig_obj).zip(&revised_offsets) {
        writer.pad_until(*revised_offset);
        writer.write(
            phdr.data(orig_endian, orig)
                .map_err(|_| anyhow!("invalid"))?,
        );
    }

    for (injection, meta) in &new_segments {
        writer.pad_until(meta.offset);
        writer.write(injection.content());
    }

    let mut recorded: Vec<(String, u64)> = vec![];
    {
        let image_start = first_vaddr(orig_obj)?;
        let image_end = new_segments
            .iter()
            .map(|(injection, _offset)| injection.vaddr() + injection.size())
            .max()
            .unwrap_or(orig_image_end)
            .try_into()?;
        for name in &image_bounds_symbols.start {
            recorded.push((name.clone(), image_start))
        }
        for name in &image_bounds_symbols.end {
            recorded.push((name.clone(), image_end))
        }
        for name in &image_bounds_symbols.size {
            recorded.push((name.clone(), image_end - image_start))
        }
    }
    for (injection, _offset) in &new_segments {
        for (name, value) in injection.witnesses() {
            recorded.push((name.clone(), *value))
        }
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

fn loadable_segments<'a>(
    obj: &'a ElfFile64,
) -> impl Iterator<Item = &'a ProgramHeader64<Endianness>> {
    obj.raw_segments()
        .iter()
        .filter(|phdr| phdr.p_type.get(obj.endian()) == PT_LOAD)
}

pub fn get_symbol_vaddr(obj: &ElfFile64, name: &str) -> Result<u64> {
    for symbol in obj.symbols() {
        if symbol.name()? == name {
            ensure!(symbol.size() == 8);
            return Ok(symbol.address());
        }
    }
    Err(anyhow!("symbol '{}' not present", name))
}

pub fn vaddr_to_offset(obj: &ElfFile64, vaddr: u64) -> Result<u64> {
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

pub fn first_vaddr(obj: &ElfFile64) -> Result<u64> {
    loadable_segments(obj)
        .map(|phdr| phdr.p_vaddr.get(obj.endian()))
        .min()
        .ok_or(anyhow!("no segments"))
}

pub fn next_vaddr(obj: &ElfFile64) -> Result<u64> {
    loadable_segments(obj)
        .map(|phdr| phdr.p_vaddr.get(obj.endian()) + phdr.p_memsz.get(obj.endian()))
        .max()
        .ok_or(anyhow!("no segments"))
}
