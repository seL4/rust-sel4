use std::fs;

use anyhow::Result;

use crate::args::*;

pub const DEFAULT_ALIGN: usize = 4096;

pub struct Injection {
    vaddr: usize,
    content: Vec<u8>,
    witnesses: Vec<(WitnessName, WitnessValue)>,
}

type WitnessName = String;

type WitnessValue = u64;

pub struct SymbolicInjection {
    align: usize,
    symbolic_content: Vec<u8>,
    witnesses: Vec<(WitnessName, SymbolicWitnessValue)>,
}

#[derive(Debug)]
enum SymbolicWitnessValue {
    Addend(i64),
    Absolute(u64),
}

impl SymbolicInjection {
    pub fn new(align: usize, buf: Vec<u8>, bounds_symbols: &BoundsSymbolArgs) -> Self {
        let size = buf.len();
        Self {
            align,
            symbolic_content: buf,
            witnesses: mk_witnesses(size, bounds_symbols),
        }
    }

    pub fn from_arg(arg: &InjectionArg) -> Result<Self> {
        let buf = fs::read(&arg.file_path)?;
        Ok(Self::new(
            arg.align.unwrap_or(DEFAULT_ALIGN),
            buf,
            &arg.bounds_symbols,
        ))
    }

    pub fn size(&self) -> usize {
        self.symbolic_content.len()
    }

    pub fn align(&self) -> usize {
        self.align
    }

    pub fn locate(self, vaddr: usize) -> Result<Injection> {
        Ok(Injection {
            vaddr,
            witnesses: self
                .witnesses
                .into_iter()
                .map(|(name, symbolic_value)| {
                    (
                        name,
                        match symbolic_value {
                            SymbolicWitnessValue::Addend(addend) => {
                                u64::try_from(vaddr).unwrap() + u64::try_from(addend).unwrap()
                            }
                            SymbolicWitnessValue::Absolute(absolute) => absolute,
                        },
                    )
                })
                .collect::<Vec<(WitnessName, WitnessValue)>>(),
            content: self.symbolic_content,
        })
    }
}

impl Injection {
    pub fn size(&self) -> usize {
        self.content.len()
    }

    pub fn vaddr(&self) -> usize {
        self.vaddr
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    pub fn witnesses(&self) -> impl Iterator<Item = &(WitnessName, WitnessValue)> {
        self.witnesses.iter()
    }
}

fn mk_witnesses(
    size: usize,
    bounds_symbols: &BoundsSymbolArgs,
) -> Vec<(WitnessName, SymbolicWitnessValue)> {
    let mut witnesses = vec![];
    for name in &bounds_symbols.start {
        witnesses.push((name.clone(), SymbolicWitnessValue::Addend(0)))
    }
    for name in &bounds_symbols.end {
        witnesses.push((
            name.clone(),
            SymbolicWitnessValue::Addend(size.try_into().unwrap()),
        ))
    }
    for name in &bounds_symbols.size {
        witnesses.push((
            name.clone(),
            SymbolicWitnessValue::Absolute(size.try_into().unwrap()),
        ))
    }
    witnesses
}
