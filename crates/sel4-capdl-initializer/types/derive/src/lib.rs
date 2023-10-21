//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(IsCap)]
pub fn derive_cap(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_cap_impl(&ast)
}

fn derive_cap_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl<'b> TryFrom<&'b Cap> for &'b #name {
            type Error = TryFromCapError;
            fn try_from(cap: &'b Cap) -> Result<Self, Self::Error> {
                match cap {
                    Cap::#name(cap) => Ok(&cap),
                    _ => Err(TryFromCapError),
                }
            }
        }
        impl Into<Cap> for #name {
            fn into(self) -> Cap {
                Cap::#name(self)
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(IsObject)]
pub fn derive_object(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_object_impl(&ast)
}

fn derive_object_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let gen = quote! {
        impl<'a, 'b, D, M> TryFrom<&'b Object<'a, D, M>> for &'b #name #generics {
            type Error = TryFromObjectError;
            fn try_from(obj: &'b Object<'a, D, M>) -> Result<Self, Self::Error> {
                match obj {
                    Object::#name(cap) => Ok(&cap),
                    _ => Err(TryFromObjectError),
                }
            }
        }
        impl<'a, D, M> Into<Object<'a, D, M>> for #name #generics {
            fn into(self) -> Object<'a, D, M> {
                Object::#name(self)
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(IsObjectWithCapTable)]
pub fn derive_object_with_cap_table(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    derive_object_with_cap_table_impl(&ast)
}

fn derive_object_with_cap_table_impl(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let generics = &ast.generics;
    let gen = quote! {
        impl #generics HasCapTable for #name #generics {
            fn slots(&self) -> &[CapTableEntry] {
                &*self.slots
            }
        }
    };
    gen.into()
}
