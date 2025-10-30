//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_derive(IsCap)]
pub fn derive_cap(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_cap_impl(&ast)
}

fn derive_cap_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let archived_name = format_ident!("Archived{}", name);
    quote! {
        impl IsCap for #name {
            fn into_cap(self) -> Cap {
                Cap::#name(self)
            }

            fn try_from_cap(cap: &Cap) -> Option<&Self> {
                match cap {
                    Cap::#name(sub_cap) => Some(&sub_cap),
                    _ => None,
                }
            }
        }

        impl IsArchivedCap for #archived_name {
            fn try_from_cap(cap: &ArchivedCap) -> Option<&Self> {
                match cap {
                    ArchivedCap::#name(sub_cap) => Some(&sub_cap),
                    _ => None,
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(IsObject)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_object_impl(&ast)
}

fn derive_object_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let archived_name = format_ident!("Archived{}", name);
    let generics = &ast.generics;
    quote! {
        impl<D> IsObject<D> for #name #generics {
            fn into_object(self) -> Object<D> {
                Object::#name(self)
            }

            fn try_from_object(obj: &Object<D>) -> Option<&Self> {
                match obj {
                    Object::#name(sub_obj) => Some(&sub_obj),
                    _ => None,
                }
            }
        }

        impl<D: Archive> IsArchivedObject<D> for #archived_name #generics {
            fn try_from_object(obj: &ArchivedObject<D>) -> Option<&Self> {
                match obj {
                    ArchivedObject::#name(sub_obj) => Some(&sub_obj),
                    _ => None,
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(HasCapTable)]
pub fn derive_has_cap_table(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_has_cap_table_impl(&ast)
}

fn derive_has_cap_table_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let archived_name = format_ident!("Archived{}", name);
    let generics = &ast.generics;
    quote! {
        impl #generics HasCapTable for #name #generics {
            fn slots(&self) -> &[CapTableEntry] {
                &*self.slots
            }
        }

        impl #generics HasArchivedCapTable for #archived_name #generics {
            fn slots(&self) -> &[ArchivedCapTableEntry] {
                &*self.slots
            }
        }
    }
    .into()
}
