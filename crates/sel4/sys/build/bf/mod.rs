use std::fs;
use std::path::Path;

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, IdentFragment};
use syn::Ident;

mod simplified;
use simplified::*;

pub fn generate_rust(
    blocklist_for_bindgen: &mut Vec<String>,
    bf_path: impl AsRef<Path>,
) -> (TokenStream, TokenStream) {
    let text = fs::read_to_string(bf_path).unwrap();
    let file = sel4_bitfield_parser::parse(&text);
    let file = simplify(&file);
    let mut generator = BitfieldGenerator::new(blocklist_for_bindgen);
    for block in file.blocks.iter() {
        generator.generate_block(&block.name, &block.backing_type, &block.fields, None);
    }
    for tagged_union in file.tagged_unions.iter() {
        generator.generate_tagged_union(tagged_union);
    }
    (generator.native_toks, generator.wrapper_toks)
}

struct BitfieldGenerator<'a> {
    blocklist_for_bindgen: &'a mut Vec<String>,
    native_toks: TokenStream,
    wrapper_toks: TokenStream,
}

impl<'a> BitfieldGenerator<'a> {
    fn new(blocklist_for_bindgen: &'a mut Vec<String>) -> Self {
        Self {
            blocklist_for_bindgen,
            native_toks: quote!(),
            wrapper_toks: quote!(),
        }
    }

    fn generate_block(
        &mut self,
        name: &str,
        backing_type: &BackingType,
        fields: &[Field],
        tag_info: Option<BlockTagInfo>,
    ) {
        let name_ident = match tag_info.as_ref() {
            Some(tag_info) => {
                mk_tagged_union_variant_block_type_ident(&tag_info.tagged_union_name, name)
            }
            None => format_ident!("{}", name),
        };

        self.blocklist_for_bindgen.push(name_ident.to_string());

        let qualified_name = quote!(crate::#name_ident);
        let unpacked_ident = format_ident!("{}_Unpacked", name_ident);

        let primitive_type = backing_type.primitive();
        let bitfield_type = backing_type.bitfield();

        let mut non_tag_fields_with_types = vec![];
        let mut non_tag_fields = vec![];
        let mut unpack_field_assignments = vec![];
        let mut new_body = quote!();
        let mut methods = quote!();
        let mut wrapper_functions = quote!();

        for field in fields.iter() {
            let field_name_ident = format_ident!("{}", field.name);
            let get_method_ident = format_ident!("get_{}", field.name);
            let set_method_ident = format_ident!("set_{}", field.name);
            let width_method_ident = format_ident!("width_of_{}", field.name);
            let field_range_start = field.offset;
            let field_range_end = field.offset + field.width;

            let tag_info_for_this_field = tag_info
                .as_ref()
                .filter(|tag_info| tag_info.tag_name == field.name);
            let is_tag = tag_info_for_this_field.is_some();

            if let Some(tag_info_for_this_field) = tag_info_for_this_field {
                let tag_values_module_ident =
                    mk_tag_values_module_ident(&tag_info_for_this_field.tagged_union_name);
                let tag_value_ident = mk_tagged_union_variant_block_type_ident(
                    &tag_info_for_this_field.tagged_union_name,
                    name,
                );
                new_body.extend(quote! {
                    this.#set_method_ident(#tag_values_module_ident::#tag_value_ident);
                });
            } else {
                non_tag_fields_with_types.push(quote! {
                    #field_name_ident: #primitive_type
                });
                non_tag_fields.push(quote! {
                    #field_name_ident
                });
                unpack_field_assignments.push(quote! {
                    #field_name_ident: self.#get_method_ident()
                });
                new_body.extend(quote! {
                    this.#set_method_ident(#field_name_ident);
                });
            };

            let visibility = if is_tag { quote!() } else { quote!(pub) };

            methods.extend(quote! {
                #[allow(dead_code)]
                #visibility fn #get_method_ident(&self) -> #primitive_type {
                    self.0.get_bits(#field_range_start..#field_range_end)
                }
                #visibility fn #set_method_ident(&mut self, #field_name_ident: #primitive_type) {
                    self.0.set_bits(#field_range_start..#field_range_end, #field_name_ident)
                }
                #[allow(dead_code)]
                #visibility const fn #width_method_ident() -> usize {
                    #field_range_end - #field_range_start
                }
            });

            let wrapper_get_prefix =
                mk_wrapper_prefix(format!("{}_get_{}", name_ident, field.name));
            let wrapper_set_prefix =
                mk_wrapper_prefix(format!("{}_set_{}", name_ident, field.name));
            let wrapper_ptr_get_prefix =
                mk_wrapper_prefix(format!("{}_ptr_get_{}", name_ident, field.name));
            let wrapper_ptr_set_prefix =
                mk_wrapper_prefix(format!("{}_ptr_set_{}", name_ident, field.name));

            if !is_tag {
                wrapper_functions.extend(quote! {
                    #wrapper_get_prefix(this: #qualified_name) -> #primitive_type {
                        this.#get_method_ident()
                    }
                    #wrapper_set_prefix(mut this: #qualified_name, #field_name_ident: #primitive_type) -> #qualified_name {
                        this.#set_method_ident(#field_name_ident);
                        this
                    }
                    #wrapper_ptr_get_prefix(this: *mut #qualified_name) -> #primitive_type {
                        unsafe {
                            (&*this).#get_method_ident()
                        }
                    }
                    #wrapper_ptr_set_prefix(this: *mut #qualified_name, #field_name_ident: #primitive_type) {
                        unsafe {
                            (&mut *this).#set_method_ident(#field_name_ident);
                        }
                    }
                })
            }
        }

        let alias_stmt = if tag_info.is_none() {
            // used by code generated by bindgen
            let alias_t = format_ident!("{}_t", name);
            quote! {
                pub type #alias_t = #name_ident;
            }
        } else {
            quote!()
        };

        self.native_toks.extend(quote! {
            #[repr(transparent)]
            #[derive(Clone, Eq, PartialEq)]
            pub struct #name_ident(pub #bitfield_type);

            #alias_stmt

            impl #name_ident {
                pub fn new(#(#non_tag_fields_with_types,)*) -> Self {
                    let mut this = Self(Bitfield::zeroed());
                    #new_body
                    this
                }

                pub fn unpack(&self) -> #unpacked_ident {
                    #unpacked_ident {
                        #(#unpack_field_assignments),*
                    }
                }

                #methods
            }

            impl fmt::Debug for #name_ident {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.unpack().fmt(f)?;
                    write!(f, ".pack()")?;
                    Ok(())
                }
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub struct #unpacked_ident {
                #(pub #non_tag_fields_with_types,)*
            }

            impl #unpacked_ident {
                pub fn pack(self) -> #name_ident {
                    match self {
                        Self { #(#non_tag_fields,)* } => #name_ident::new(#(#non_tag_fields,)*),
                    }
                }
            }
        });

        let wrapper_new_prefix = mk_wrapper_prefix(format!("{name_ident}_new"));
        let wrapper_ptr_new_prefix = mk_wrapper_prefix(format!("{name_ident}_ptr_new"));

        self.wrapper_toks.extend(quote! {
            #wrapper_new_prefix(#(#non_tag_fields_with_types,)*) -> #qualified_name {
                #qualified_name::new(#(#non_tag_fields,)*)
            }

            #wrapper_ptr_new_prefix(this: *mut #qualified_name, #(#non_tag_fields_with_types,)*) {
                unsafe {
                    *this = #qualified_name::new(#(#non_tag_fields,)*);
                }
            }

            #wrapper_functions
        });
    }

    fn generate_tagged_union(&mut self, tagged_union: &TaggedUnion) {
        let name_ident = format_ident!("{}", tagged_union.name);
        let qualified_name = quote!(crate::#name_ident);
        let splayed_ident = format_ident!("{}_Splayed", tagged_union.name);
        let primitive_type = tagged_union.backing_type.primitive();
        let bitfield_type = tagged_union.backing_type.bitfield();
        let tag_values_module_ident = mk_tag_values_module_ident(&tagged_union.name);

        self.blocklist_for_bindgen.push(name_ident.to_string());
        self.blocklist_for_bindgen
            .push(tag_values_module_ident.to_string());
        self.blocklist_for_bindgen
            .push(tagged_union.tag_name.to_owned()); // prevent bindgen from emitting code like "pub use self::seL4_Fault_tag_t as seL4_FaultType;"

        let mut tag_value_consts = vec![];
        let mut splayed_variants = vec![];
        let mut splay_match_arms = vec![];
        let mut unsplay_match_arms = vec![];
        let mut block_unsplay_toks = quote!();

        for tag in tagged_union.tags.iter() {
            let tag_name_ident = format_ident!("{}", tag.name);
            let splayed_variant = tag_name_ident.clone();
            let block_type =
                mk_tagged_union_variant_block_type_ident(&tagged_union.name, &tag.name);
            let tag_value_ident = block_type.clone();
            let tag_value = Literal::u128_unsuffixed(tag.value.try_into().unwrap()); // proc_macro2 doesn't have a generic unsuffixed integer literal type
            let unpacked_ident = format_ident!("{}_{}_Unpacked", name_ident, tag.name);

            tag_value_consts.push(quote! {
                pub const #tag_value_ident: #primitive_type = #tag_value
            });
            splayed_variants.push(quote! {
                #splayed_variant(#block_type)
            });
            splay_match_arms.push(quote! {
                #tag_values_module_ident::#tag_value_ident => #splayed_ident::#splayed_variant(#block_type(self.0))
            });
            unsplay_match_arms.push(quote! {
                #splayed_ident::#splayed_variant(#block_type(bitfield)) => #name_ident(bitfield),
            });

            self.generate_block(
                &tag.name,
                &tagged_union.backing_type,
                &tag.fields,
                Some(BlockTagInfo {
                    tagged_union_name: tagged_union.name.clone(),
                    tag_name: tagged_union.tag_name.clone(),
                }),
            );

            block_unsplay_toks.extend(quote! {
                impl #block_type {
                    pub fn unsplay(self) -> #name_ident {
                        #name_ident(self.0)
                    }
                }
                impl #unpacked_ident {
                    pub fn unsplay(self) -> #name_ident {
                        self.pack().unsplay()
                    }
                }
            });
        }

        let tag_range_start = tagged_union.tag_range.start;
        let tag_range_end = tagged_union.tag_range.end;

        self.native_toks.extend(quote! {
            pub mod #tag_values_module_ident {
                #(#tag_value_consts;)*
            }

            #[repr(transparent)]
            #[derive(Clone, PartialEq, Eq)]
            pub struct #name_ident(pub #bitfield_type);

            impl #name_ident {
                pub fn splay(self) -> #splayed_ident {
                    match self.get_tag() {
                        #(#splay_match_arms,)*
                        _ => panic!(),
                    }
                }

                pub fn get_tag(&self) -> #primitive_type {
                    self.0.get_bits(#tag_range_start..#tag_range_end)
                }
            }

            impl fmt::Debug for #name_ident {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.clone().splay().fmt(f)?;
                    write!(f, ".unsplay()")?;
                    Ok(())
                }
            }

            #[derive(Debug, Clone, PartialEq, Eq)]
            pub enum #splayed_ident {
                #(#splayed_variants,)*
            }

            impl #splayed_ident {
                pub fn unsplay(self) -> #name_ident {
                    match self {
                        #(#unsplay_match_arms)*
                    }
                }
            }

            #block_unsplay_toks
        });

        let wrapper_get_tag_prefix = mk_wrapper_prefix(format!(
            "{}_get_{}",
            tagged_union.name, tagged_union.tag_name
        ));
        let wrapper_ptr_get_tag_prefix = mk_wrapper_prefix(format!(
            "{}_ptr_get_{}",
            tagged_union.name, tagged_union.tag_name
        ));
        let wrapper_tag_equals_prefix = mk_wrapper_prefix(format!(
            "{}_{}_equals",
            tagged_union.name, tagged_union.tag_name
        ));

        let c_int = quote!(::core::ffi::c_int);

        self.wrapper_toks.extend(quote! {
            #wrapper_get_tag_prefix(this: #qualified_name) -> #primitive_type {
                this.get_tag()
            }

            #wrapper_ptr_get_tag_prefix(this: *mut #qualified_name) -> #primitive_type {
                unsafe {
                    (&*this).get_tag()
                }
            }

            #wrapper_tag_equals_prefix(this: #qualified_name, tag: #primitive_type) -> #c_int {
                (this.get_tag() == tag) as #c_int
            }
        });
    }
}

struct BlockTagInfo {
    tagged_union_name: String,
    tag_name: String,
}

impl BackingType {
    fn primitive(&self) -> TokenStream {
        format!("u{}", self.base).parse::<TokenStream>().unwrap()
    }

    fn bitfield(&self) -> TokenStream {
        let primitive = self.primitive();
        let multiple = self.multiple;
        quote!(SeL4Bitfield<#primitive, #multiple>)
    }
}

fn mk_tagged_union_variant_block_type_ident(tagged_union_name: &str, tag_name: &str) -> Ident {
    format_ident!("{}_{}", tagged_union_name, tag_name)
}

fn mk_tag_values_module_ident(tagged_union_name: &str) -> Ident {
    format_ident!("{}_tag", tagged_union_name)
}

fn mk_wrapper_prefix(fn_name: impl IdentFragment) -> TokenStream {
    let fn_ident = format_ident!("{}", fn_name);
    quote! {
        #[no_mangle]
        pub extern "C" fn #fn_ident
    }
}
