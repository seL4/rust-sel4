#![feature(int_roundings)]

use anyhow::Result;

mod render;

// NOTE
// The phdrs in output of render_with_data have p_align=1 regardless of the input.
// That is because the current consumers of the output do not use p_align.

#[derive(Default)]
pub struct Input<'a> {
    pub symbolic_injections: Vec<SymbolicInjection<'a>>,
    pub image_start_patches: Vec<Symbol>,
    pub image_end_patches: Vec<Symbol>,
    pub concrete_patches: Vec<(Symbol, ConcreteValue)>,
}

type Symbol = String;

type ConcreteValue = u64;

pub struct SymbolicInjection<'a> {
    pub align_modulus: usize,
    pub align_residue: usize,
    pub content: &'a [u8],
    pub memsz: usize,
    pub patches: Vec<(Symbol, SymbolicValue)>,
}

#[derive(Debug)]
pub struct SymbolicValue {
    pub addend: i64,
}

impl<'a> SymbolicInjection<'a> {
    fn filesz(&self) -> usize {
        self.content.len()
    }

    fn align_from(&self, addr: usize) -> usize {
        align_from(addr, self.align_modulus, self.align_residue)
    }

    fn locate(&self, vaddr: usize) -> Result<Injection<'a>> {
        Ok(Injection {
            vaddr,
            content: self.content,
            memsz: self.memsz,
            patches: self
                .patches
                .iter()
                .map(|(name, symbolic_value)| {
                    (
                        name.clone(),
                        u64::try_from(vaddr).unwrap()
                            + u64::try_from(symbolic_value.addend).unwrap(),
                    )
                })
                .collect::<Vec<(Symbol, ConcreteValue)>>(),
        })
    }
}

pub struct Injection<'a> {
    pub vaddr: usize,
    pub content: &'a [u8],
    pub memsz: usize,
    pub patches: Vec<(Symbol, ConcreteValue)>,
}

impl<'a> Injection<'a> {
    fn vaddr(&self) -> usize {
        self.vaddr
    }

    fn filesz(&self) -> usize {
        self.content.len()
    }

    fn memsz(&self) -> usize {
        self.memsz
    }

    fn content(&self) -> &'a [u8] {
        self.content
    }

    fn patches(&self) -> impl Iterator<Item = &(Symbol, ConcreteValue)> {
        self.patches.iter()
    }
}

fn align_from(addr: usize, modulus: usize, residue: usize) -> usize {
    addr + (modulus + residue - addr % modulus) % modulus
}
