use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};

use crate::scheme::{Scheme, SchemeHelpers, SchemeLeafDescriptor};
use crate::table::{AbstractEntry, Table};

impl<T: Scheme> Table<T> {
    pub fn embed(&self, ident: Ident) -> TokenStream
    where
        T::WordPrimitive: ToTokens,
    {
        Embedding::new(ident).embed(self)
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

    fn embed<'a, T: Scheme>(mut self, table: &'a Table<T>) -> TokenStream {
        let _ = self.embed_inner(table);
        self.check_tables_order();
        let ident = self.ident;
        let num_tables = self.next_index;
        let num_entries = SchemeHelpers::<T>::num_entries_in_table();
        let tables = self.tables.values();
        quote! {
            #[repr(C, align(4096))]
            pub struct Table(pub [*const (); #num_entries]);

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

    fn embed_inner<'a, T: Scheme>(&mut self, table: &'a Table<T>) -> usize {
        let index = self.allocate_index();
        let entries = table.entries.iter().map(|entry| {
            let entry = EntryForEmbedding::<T>::from_abstract_entry(entry);
            let offset = entry.offset;
            match &entry.ptr {
                None => {
                    quote! {
                        (#offset as *const ())
                    }
                }
                Some(ptr) => {
                    let child_index = self.embed_inner(ptr);
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
