use std::fs;
use std::ops::Range;

use anyhow::{anyhow, ensure, Result};
use object::{pod, read::elf::ElfFile64, Endian, Object, ObjectSegment, ObjectSymbol};

mod args;

use args::{Args, SymbolNames};

pub fn main() -> Result<()> {
    let args = Args::parse()?;
    if args.verbose {
        eprintln!("{:#?}", args);
    }
    let mut buf = fs::read(args.in_file_path)?;
    inject_phdrs(buf.as_mut_slice(), &args.symbols_names)?;
    fs::write(args.out_file_path, &buf)?;
    Ok(())
}

pub fn inject_phdrs(buf: &mut [u8], symbol_names: &SymbolNames) -> Result<()> {
    let obj: &ElfFile64 = &ElfFile64::parse(&*buf)?;
    let endian = obj.endian();
    let phdrs = obj.raw_segments().to_vec();
    let num_phdrs_offset = symbol_offset(obj, &symbol_names.num_phdrs)?;
    let phdrs_offset = symbol_offset(obj, &symbol_names.phdrs)?;
    buf[num_phdrs_offset]
        .copy_from_slice(&endian.write_u64_bytes(phdrs.len().try_into().unwrap())[..]);
    let phdrs_bytes = pod::bytes_of_slice(&phdrs);
    buf[phdrs_offset][..phdrs_bytes.len()].copy_from_slice(phdrs_bytes);
    Ok(())
}

pub fn symbol_offset(obj: &ElfFile64, name: &str) -> Result<Range<usize>> {
    let vaddr = symbol_vaddr(obj, name)?;
    let offset = vaddr_to_offset(obj, vaddr.start)?..vaddr_to_offset(obj, vaddr.end)?;
    Ok(offset)
}

pub fn symbol_vaddr(obj: &ElfFile64, name: &str) -> Result<Range<u64>> {
    for symbol in obj.symbols() {
        if symbol.name()? == name {
            return Ok(symbol.address()..symbol.address() + symbol.size());
        }
    }
    Err(anyhow!("symbol '{}' not present", name))
}

pub fn vaddr_to_offset(obj: &ElfFile64, vaddr: u64) -> Result<usize> {
    for segment in obj.segments() {
        let start = segment.address();
        let end = start + segment.size();
        if (start..end).contains(&vaddr) {
            let offset_in_segment = vaddr - start;
            let (file_start, file_size) = segment.file_range();
            ensure!(offset_in_segment <= file_size);
            return Ok((file_start + offset_in_segment).try_into().unwrap());
        }
    }
    Err(anyhow!("vaddr '0x{:x}' not mapped", vaddr))
}
