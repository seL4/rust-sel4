//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, format_ident, quote};

use crate::scheme::{Scheme, SchemeLeafDescriptor};
use crate::table::{AbstractEntry, Table};

impl<T: Scheme> Table<T> {
    pub fn embed(&self, symbol_ident: Ident, runtime_mod_ident: Ident) -> TokenStream
    where
        T::WordPrimitive: ToTokens,
    {
        Embedding::new(symbol_ident, runtime_mod_ident).embed(self)
    }
}

struct EntryForEmbedding<'a, T: Scheme> {
    offset: T::WordPrimitive,
    ptr: Option<&'a Table<T>>,
}

impl<'a, T: Scheme> EntryForEmbedding<'a, T> {
    fn from_abstract_entry(entry: &'a AbstractEntry<T>) -> Self {
        match entry {
            AbstractEntry::Empty => Self {
                offset: T::EMPTY_DESCRIPTOR,
                ptr: None,
            },
            AbstractEntry::Leaf(descriptor) => Self {
                offset: descriptor.to_raw(),
                ptr: None,
            },
            AbstractEntry::Branch(branch) => Self {
                offset: T::SYMBOLIC_BRANCH_DESCRIPTOR_OFFSET,
                ptr: Some(branch),
            },
        }
    }
}

struct Embedding {
    symbol_ident: Ident,
    runtime_mod_ident: Ident,
    next_index: usize,
    tables: BTreeMap<usize, TokenStream>,
}

impl Embedding {
    fn new(symbol_ident: Ident, runtime_mod_ident: Ident) -> Self {
        Self {
            symbol_ident,
            runtime_mod_ident,
            next_index: 0,
            tables: BTreeMap::new(),
        }
    }

    fn embed<T: Scheme>(mut self, table: &Table<T>) -> TokenStream {
        let _ = self.embed_inner(table, 0);
        self.check_tables_order();
        let runtime_mod_ident = self.runtime_mod_ident;
        let symbol_ident = format_ident!("{}", self.symbol_ident);
        let symbol_access_ident = format_ident!("{}_access", self.symbol_ident);
        let tables = self.tables.values();
        quote! {
            use #runtime_mod_ident::*;

            #[allow(non_upper_case_globals)]
            pub static #symbol_access_ident: TablePtrs = TablePtrs::new(&[
                #(#tables,)*
            ]);

            #[unsafe(no_mangle)]
            #[allow(unused_unsafe)]
            pub static #symbol_ident: TablePtr = #symbol_access_ident.root();
        }
    }

    fn check_tables_order(&self) {
        self.tables.keys().enumerate().for_each(|(i, k)| {
            assert_eq!(i, *k);
        });
    }

    fn embed_inner<T: Scheme>(&mut self, table: &Table<T>, level: usize) -> usize {
        let index = self.allocate_index();
        let entries = table.entries.iter().map(|entry| {
            let entry = EntryForEmbedding::<T>::from_abstract_entry(entry);
            let ptr = match &entry.ptr {
                None => {
                    quote! {
                        None
                    }
                }
                Some(ptr) => {
                    let child_index = self.embed_inner(ptr, level + 1);
                    let symbol_access_ident = format_ident!("{}_access", self.symbol_ident);
                    quote! {
                        Some(#symbol_access_ident.table(#child_index).value())
                    }
                }
            };
            let offset = entry.offset;
            quote! {
                Entry::new(#ptr, #offset as usize)
            }
        });
        let align_type = format_ident!("A{}", 1usize << T::level_align_bits(level));
        let num_entries = table.entries.len();
        let toks = quote! {
            {
                static TABLE: Table<#align_type, #num_entries> = Table::new([#(#entries,)*]);
                TABLE.ptr()
            }
        };
        self.tables.insert(index, toks);
        index
    }

    fn allocate_index(&mut self) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        index
    }
}
