extern crate proc_macro;

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
        impl<'a> TryFrom<&'a Cap> for &'a #name {
            type Error = TryFromCapError;
            fn try_from(obj: &'a Cap) -> Result<Self, Self::Error> {
                match obj {
                    Cap::#name(obj) => Ok(&obj),
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
        impl<'a, C, F> TryFrom<&'a Object<C, F>> for &'a #name #generics {
            type Error = TryFromObjectError;
            fn try_from(obj: &'a Object<C, F>) -> Result<Self, Self::Error> {
                match obj {
                    Object::#name(cap) => Ok(&cap),
                    _ => Err(TryFromObjectError),
                }
            }
        }
        impl<C, F> Into<Object<C, F>> for #name #generics {
            fn into(self) -> Object<C, F> {
                Object::#name(self)
            }
        }
    };
    gen.into()
}
