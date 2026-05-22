//
// Copyright 2026, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::fs;

use anyhow::Result;
use clap::Parser;
use object::elf::{FileHeader64, R_AARCH64_RELATIVE, Rela64};
use object::read::elf::ElfFile;
use object::{Endian, Endianness, Object, ObjectSection, ObjectSegment, pod};

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    loader: String,
    #[arg(long)]
    payload: String,
    #[arg(long, short = 'o')]
    out_file: String,
    #[arg(long, short = 'v')]
    verbose: bool,
}

const STACK_SIZE: usize = 4096 * 4; // one page is enough for --release
const STACK_ALIGNMENT: usize = 16;
const PAYLOAD_ALIGNMENT: usize = WORD_SIZE_BYTES;
const WORD_SIZE_BYTES: usize = 8;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let loader_elf_bytes = fs::read(&cli.loader)?;
    let loader_elf = ElfFile::<FileHeader64<Endianness>>::parse(&loader_elf_bytes).unwrap();
    check_relocations(&loader_elf);
    let loader_segment = loader_elf.segments().next().unwrap();
    assert_eq!(
        loader_segment.size(),
        loader_segment.data().unwrap().len().try_into().unwrap()
    );

    let endian = loader_elf.endian();

    let payload_elf_bytes = fs::read(&cli.payload)?;
    let payload_elf = ElfFile::<FileHeader64<Endianness>>::parse(&payload_elf_bytes).unwrap();
    let mut payload = Payload::new(payload_elf.entry());
    for seg in payload_elf.segments() {
        payload.add_segment(seg);
    }

    let mut buf = loader_segment.data().unwrap().to_owned();
    buf.resize(buf.len().next_multiple_of(PAYLOAD_ALIGNMENT), 0);
    buf.extend_from_slice(&payload.serialize(endian));

    let total_size = (buf.len() + STACK_SIZE).next_multiple_of(STACK_ALIGNMENT);

    buf[2 * WORD_SIZE_BYTES..][..WORD_SIZE_BYTES]
        .copy_from_slice(&u64::try_from(total_size).unwrap().to_le_bytes());

    fs::write(&cli.out_file, &buf)?;

    Ok(())
}

fn check_relocations<E: Endian>(elf: &ElfFile<FileHeader64<E>>) {
    for section in elf.sections() {
        let section_name = section.name().unwrap();
        if section_name == ".rela.dyn" {
            let relas = pod::slice_from_all_bytes::<Rela64<E>>(section.data().unwrap()).unwrap();
            for rela in relas {
                let r_type = rela.r_type(elf.endian(), false);
                if r_type != R_AARCH64_RELATIVE {
                    panic!("unsupported relocation type {} in {}", r_type, section_name);
                }
            }
        }
    }
}

struct Payload {
    entry: u64,
    regions: Vec<Region>,
    data: Vec<u8>,
}

struct Region {
    vaddr: u64,
    offset: u64,
    filesz: u64,
    memsz: u64,
}

impl Payload {
    fn new(entry: u64) -> Self {
        Self {
            entry,
            regions: vec![],
            data: vec![],
        }
    }

    fn add_segment<'a>(&mut self, seg: impl ObjectSegment<'a>) {
        let offset = self.data.len();
        let seg_data = seg.data().unwrap();
        self.data.extend_from_slice(seg_data);
        self.regions.push(Region {
            vaddr: seg.address(),
            offset: offset.try_into().unwrap(),
            filesz: seg_data.len().try_into().unwrap(),
            memsz: seg.size(),
        })
    }

    fn serialize(self, endian: impl Endian) -> Vec<u8> {
        let mut buf = vec![];
        buf.extend_from_slice(&endian.write_u64(self.entry));
        buf.extend_from_slice(&endian.write_u64(self.regions.len().try_into().unwrap()));
        for region in self.regions.iter() {
            buf.extend_from_slice(&endian.write_u64(region.vaddr));
            buf.extend_from_slice(&endian.write_u64(region.offset));
            buf.extend_from_slice(&endian.write_u64(region.filesz));
            buf.extend_from_slice(&endian.write_u64(region.memsz));
        }
        buf.extend_from_slice(&self.data);
        buf
    }
}
