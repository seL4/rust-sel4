//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::scheme::{Scheme, SchemeHelpers, SchemeLeafDescriptor};
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
        let _ = self.embed_inner(table);
        self.check_tables_order();
        let runtime_mod_ident = self.runtime_mod_ident;
        let symbol_ident = self.symbol_ident;
        let runtime_scheme_ident = format_ident!("{}", T::RUNTIME_SCHEME_IDENT);
        let num_tables = self.next_index;
        let num_entries = SchemeHelpers::<T>::num_entries_in_table();
        let tables = self.tables.values();
        quote! {
            use #runtime_mod_ident::*;

            #[no_mangle]
            #[allow(unused_unsafe)]
            pub static mut #symbol_ident: Tables<#runtime_scheme_ident, #num_entries, #num_tables> = Tables::new(unsafe {
                [#(#tables,)*]
            });
        }
    }

    fn check_tables_order(&self) {
        self.tables.keys().enumerate().for_each(|(i, k)| {
            assert_eq!(i, *k);
        });
    }

    fn embed_inner<T: Scheme>(&mut self, table: &Table<T>) -> usize {
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
                    let child_index = self.embed_inner(ptr);
                    let symbol_ident = &self.symbol_ident;
                    quote! {
                        Some(#symbol_ident.table(#child_index))
                    }
                }
            };
            let offset = entry.offset;
            quote! {
                Entry::new(#ptr, #offset as usize)
            }
        });
        let toks = quote! {
            Table::new([#(#entries,)*])
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
