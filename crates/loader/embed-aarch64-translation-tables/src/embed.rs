use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::table::{Table, NUM_ENTRIES};

impl Table {
    pub fn embed(&self, ident: Ident) -> TokenStream {
        let mut embedding = Embedding::new(&ident);
        let _ = embedding.embed(self);
        let num_tables = embedding.next_index;
        let tables = embedding.tables.iter().enumerate().map(|(i, (k, v))| {
            assert_eq!(i, *k);
            v
        });
        quote! {
            #[repr(C, align(4096))]
            pub struct Table([*const (); #NUM_ENTRIES]);

            #[no_mangle]
            #[allow(unused_unsafe)]
            pub static mut #ident: [Table; #num_tables] = unsafe {
                [#(#tables,)*]
            };
        }
    }
}

struct Embedding<'a> {
    ident: &'a Ident,
    next_index: usize,
    tables: BTreeMap<usize, TokenStream>,
}

impl<'a> Embedding<'a> {
    fn new(ident: &'a Ident) -> Self {
        Self {
            ident,
            next_index: 0,
            tables: BTreeMap::new(),
        }
    }

    fn allocate_index(&mut self) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        index
    }

    fn embed(&mut self, table: &Table) -> usize {
        let ident = self.ident;
        let index = self.allocate_index();
        let entries = table.entries.iter().map(|entry| {
            let value = entry.value;
            match &entry.child {
                None => {
                    quote! {
                        (#value as *const ())
                    }
                }
                Some(table) => {
                    let child_index = self.embed(&table);
                    quote! {
                        (&#ident[#child_index] as *const Table as *const ()).byte_add(#value as usize)
                    }
                }
            }
        });
        let toks = quote! {
            Table([#(#entries,)*])
        };
        self.tables.insert(index, toks);
        index
    }
}
