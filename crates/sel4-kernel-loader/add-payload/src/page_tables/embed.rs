//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use super::scheme::{Level, Scheme};
use super::table::{AbstractEntry, Table};

impl Table {
    pub fn embed(&self, scheme: &Scheme, vaddr: u64) -> (Vec<u8>, u64) {
        Embedding::new(scheme, vaddr).embed(self)
    }
}

struct Embedding<'a> {
    scheme: &'a Scheme,
    start_vaddr: u64,
    buf: Vec<u8>,
}

impl<'a> Embedding<'a> {
    fn new(scheme: &'a Scheme, start_vaddr: u64) -> Self {
        Self {
            scheme,
            start_vaddr,
            buf: vec![],
        }
    }

    fn embed(mut self, table: &Table) -> (Vec<u8>, u64) {
        let root_vaddr = self.embed_inner(table, 0);
        (self.buf, root_vaddr)
    }

    fn embed_inner(&mut self, table: &Table, level: Level) -> u64 {
        let bytes = table
            .entries
            .iter()
            .flat_map(|entry| {
                self.scheme.descriptor_to_bytes(match entry {
                    AbstractEntry::Empty => self.scheme.empty_descriptor(),
                    AbstractEntry::Leaf(descriptor) => *descriptor,
                    AbstractEntry::Branch(branch) => {
                        let child_vaddr = self.embed_inner(branch, level + 1);
                        self.scheme.branch_descriptor(child_vaddr)
                    }
                })
            })
            .collect::<Vec<_>>();
        self.align(1 << self.scheme.level_align_bits(level));
        let vaddr = self.cur_vaddr();
        self.buf.extend(bytes);
        vaddr
    }

    fn cur_vaddr(&self) -> u64 {
        self.start_vaddr + u64::try_from(self.buf.len()).unwrap()
    }

    fn align(&mut self, align: u64) {
        let cur_vaddr = self.cur_vaddr();
        let aligned_vaddr = cur_vaddr.next_multiple_of(align);
        self.buf
            .resize((aligned_vaddr - self.start_vaddr).try_into().unwrap(), 0);
    }
}
