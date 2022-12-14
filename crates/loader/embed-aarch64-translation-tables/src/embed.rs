use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::table::{AbstractEntry, Table, NUM_ENTRIES};

impl<T> Table<T>
where
    for<'a> &'a AbstractEntry<T>: Into<EntryForEmbedding<'a, T>>,
{
    pub fn embed(&self, ident: Ident) -> TokenStream {
        Embedding::new(ident).embed(self)
    }
}

pub struct EntryForEmbedding<'a, T> {
    pub offset: u64,
    pub ptr: Option<&'a Table<T>>,
}

struct Embedding {
    ident: Ident,
    next_index: usize,
    tables: BTreeMap<usize, TokenStream>,
}

impl Embedding {
    fn new(ident: Ident) -> Self {
        Self {
            ident,
            next_index: 0,
            tables: BTreeMap::new(),
        }
    }

    fn embed<'a, T>(mut self, table: &'a Table<T>) -> TokenStream
    where
        &'a AbstractEntry<T>: Into<EntryForEmbedding<'a, T>>,
    {
        let _ = self.embed_inner(table);
        self.check_tables_order();
        let ident = self.ident;
        let num_tables = self.next_index;
        let tables = self.tables.values();
        quote! {
            #[repr(C, align(4096))]
            pub struct Table(pub [*const (); #NUM_ENTRIES]);

            #[no_mangle]
            #[allow(unused_unsafe)]
            pub static mut #ident: [Table; #num_tables] = unsafe {
                [#(#tables,)*]
            };
        }
    }

    fn check_tables_order(&self) {
        self.tables.keys().enumerate().for_each(|(i, k)| {
            assert_eq!(i, *k);
        });
    }

    fn embed_inner<'a, T>(&mut self, table: &'a Table<T>) -> usize
    where
        &'a AbstractEntry<T>: Into<EntryForEmbedding<'a, T>>,
    {
        let index = self.allocate_index();
        let entries = table.entries.iter().map(|entry| {
            let entry: EntryForEmbedding<T> = entry.into();
            let offset = entry.offset;
            match &entry.ptr {
                None => {
                    quote! {
                        (#offset as *const ())
                    }
                }
                Some(ptr) => {
                    let child_index = self.embed_inner(&ptr);
                    let ident = &self.ident;
                    quote! {
                        (&#ident[#child_index] as *const Table as *const ()).byte_add(#offset as usize)
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

    fn allocate_index(&mut self) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        index
    }
}
