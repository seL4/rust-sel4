//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![feature(never_type)]
#![feature(unwrap_infallible)]

// TODO(nspin)
// In a few cases, we use a local const declaration to appease the borrow checker.
// Using an exposed constructor of `Indirect` would be one way around this.

use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

use sel4_capdl_initializer_types::*;

pub struct Embedding<'a> {
    pub spec: Cow<'a, SpecForEmbedding<'a>>,
    pub fill_map: Cow<'a, FillMap>,
    pub object_names_level: ObjectNamesLevel,
    pub deflate_fill: bool,
    pub granule_size_bits: usize,
}

pub type SpecForEmbedding<'a> = Spec<'a, String, FileContentRange, Fill<'a, FileContentRange>>;

fn to_tokens_via_debug(value: impl fmt::Debug) -> TokenStream {
    format!("{:?}", value).parse::<TokenStream>().unwrap()
}

impl<'a> Embedding<'a> {
    pub fn spec(&self) -> &SpecForEmbedding<'a> {
        self.spec.borrow()
    }

    pub fn fill_map(&self) -> &FillMap {
        self.fill_map.borrow()
    }

    pub fn object_names_level(&self) -> ObjectNamesLevel {
        self.object_names_level
    }

    pub fn deflate_fill(&self) -> bool {
        self.deflate_fill
    }

    pub fn granule_size_bits(&self) -> usize {
        self.granule_size_bits
    }

    pub fn granule_size(&self) -> usize {
        1 << self.granule_size_bits()
    }

    //

    fn types_mod(&self) -> TokenStream {
        quote! { sel4_capdl_initializer_types }
    }

    fn embed_cap_table(&self, slots: &[CapTableEntry]) -> TokenStream {
        let slots = slots.iter().map(|(i, cap)| {
            let cap = to_tokens_via_debug(cap);
            quote! {
                (#i, Cap::#cap)
            }
        });
        quote! {
            {
                use cap::*;
                Indirect::from_borrowed([#(#slots,)*].as_slice())
            }
        }
    }

    fn embed_frame_init(&self, frame_init: &FrameInit<Ident, Ident>) -> TokenStream {
        match frame_init {
            FrameInit::Fill(fill) => {
                let entries = fill.entries.iter().map(|entry| {
                    let range = to_tokens_via_debug(&entry.range);
                    let content = match &entry.content {
                        FillEntryContent::Data(ident) => {
                            let outer_value = self.fill_value(ident);
                            quote! {
                                FillEntryContent::Data(#outer_value)
                            }
                        }
                        content @ FillEntryContent::BootInfo(_) => to_tokens_via_debug(content),
                    };
                    quote! {
                        FillEntry {
                            range: #range,
                            content: #content,
                        }
                    }
                });
                quote! {
                    FrameInit::Fill(Fill {
                        entries: {
                            use FillEntryContent::*;
                            use FillEntryContentBootInfoId::*;
                            Indirect::from_borrowed([#(#entries,)*].as_slice())
                        },
                    })
                }
            }
            FrameInit::Embedded(ident) => {
                quote! {
                    FrameInit::Embedded(#ident)
                }
            }
        }
    }

    fn patch_field(&self, expr_struct: &mut syn::ExprStruct, field_name: &str, value: syn::Expr) {
        for field in expr_struct.fields.iter_mut() {
            match &field.member {
                syn::Member::Named(ident) if ident == field_name => {
                    field.expr = value.clone();
                }
                _ => {}
            }
        }
    }

    fn embed_object_with_cap_table(&self, obj: &(impl fmt::Debug + HasCapTable)) -> TokenStream {
        let mut expr_struct = syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(obj)).unwrap();
        self.patch_field(
            &mut expr_struct,
            "slots",
            syn::parse2::<syn::Expr>(self.embed_cap_table(obj.slots())).unwrap(),
        );
        expr_struct.to_token_stream()
    }

    fn embed_object(&self, obj: &Object<Ident, Ident>) -> TokenStream {
        match obj {
            Object::CNode(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::CNode(object::#toks))
            }
            Object::TCB(obj) => {
                let mut expr_struct =
                    syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(obj)).unwrap();
                self.patch_field(
                    &mut expr_struct,
                    "slots",
                    syn::parse2::<syn::Expr>(self.embed_cap_table(obj.slots())).unwrap(),
                );
                self.patch_field(
                    &mut expr_struct,
                    "extra",
                    syn::parse2::<syn::Expr>({
                        let mut inner_expr_struct =
                            syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(&obj.extra))
                                .unwrap();
                        self.patch_field(
                            &mut inner_expr_struct,
                            "gprs",
                            syn::parse2::<syn::Expr>({
                                let gprs = to_tokens_via_debug(&obj.extra.gprs);
                                quote!(Indirect::from_borrowed(&#gprs))
                            })
                            .unwrap(),
                        );
                        quote! {
                            {
                                const EXTRA: &TCBExtraInfo<'static> = &#inner_expr_struct;
                                Indirect::from_borrowed(EXTRA)
                            }
                        }
                    })
                    .unwrap(),
                );
                let toks = expr_struct.to_token_stream();
                quote! {
                    {
                        use object::{TCB, TCBExtraInfo};
                        Object::TCB(#toks)
                    }
                }
            }
            Object::IRQ(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::IRQ(object::#toks))
            }
            Object::Frame(obj) => {
                let mut expr_struct =
                    syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(obj)).unwrap();
                self.patch_field(
                    &mut expr_struct,
                    "init",
                    syn::parse2::<syn::Expr>(self.embed_frame_init(&obj.init)).unwrap(),
                );
                let toks = expr_struct.to_token_stream();
                quote!(Object::Frame(object::#toks))
            }
            Object::PageTable(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::PageTable(object::#toks))
            }
            Object::ArmIRQ(obj) => {
                let mut expr_struct =
                    syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(obj)).unwrap();
                self.patch_field(
                    &mut expr_struct,
                    "slots",
                    syn::parse2::<syn::Expr>(self.embed_cap_table(obj.slots())).unwrap(),
                );
                self.patch_field(
                    &mut expr_struct,
                    "extra",
                    syn::parse2::<syn::Expr>({
                        let extra = to_tokens_via_debug(&obj.extra);
                        quote!(Indirect::from_borrowed(&#extra))
                    })
                    .unwrap(),
                );
                let toks = expr_struct.to_token_stream();
                quote! {
                    {
                        use object::{ArmIRQ, ArmIRQExtraInfo};
                        Object::ArmIRQ(#toks)
                    }
                }
            }
            obj => {
                let obj = to_tokens_via_debug(obj);
                quote! {
                    {
                        use object::*;
                        Object::#obj
                    }
                }
            }
        }
    }

    fn qualification_prefix(&self, qualify: bool) -> TokenStream {
        if qualify {
            let types_mod = self.types_mod();
            quote!(#types_mod::)
        } else {
            quote!()
        }
    }

    fn name_type(&self, qualify: bool) -> TokenStream {
        let prefix = self.qualification_prefix(qualify);
        let inner = match self.object_names_level() {
            ObjectNamesLevel::All => quote!(&str),
            ObjectNamesLevel::JustTCBs => quote!(Option<&str>),
            ObjectNamesLevel::None => quote!(#prefix Unnamed),
        };
        quote!(#prefix SelfContained<#inner>)
    }

    fn name_value<D, M>(&self, obj: &Object<'a, D, M>, name: &str) -> TokenStream {
        let inner = match self.object_names_level() {
            ObjectNamesLevel::All => quote!(#name),
            ObjectNamesLevel::JustTCBs => match obj {
                Object::TCB(_) => quote!(Some(#name)),
                _ => quote!(None),
            },
            ObjectNamesLevel::None => quote!(Unnamed),
        };
        quote!(SelfContained::new(#inner))
    }

    fn fill_type_inner(&self, qualify: bool) -> TokenStream {
        let prefix = self.qualification_prefix(qualify);
        let ty = if self.deflate_fill() {
            quote!(DeflatedBytesContent)
        } else {
            quote!(BytesContent)
        };
        quote!(#prefix #ty)
    }

    fn fill_type(&self, qualify: bool) -> TokenStream {
        let prefix = self.qualification_prefix(qualify);
        let inner = self.fill_type_inner(qualify);
        quote!(#prefix SelfContained<#inner>)
    }

    fn fill_value(&self, field_value: impl ToTokens) -> TokenStream {
        let constructor = self.fill_type_inner(false);
        let field_name = if self.deflate_fill() {
            quote!(deflated_bytes)
        } else {
            quote!(bytes)
        };
        quote! {
            SelfContained::new(#constructor {
                #field_name: #field_value
            })
        }
    }

    fn pack_fill(&self, bytes: &[u8]) -> Vec<u8> {
        (if self.deflate_fill() {
            DeflatedBytesContent::pack
        } else {
            BytesContent::pack
        })(bytes)
    }

    // NOTE
    // I would prefer this to return just the rhs, but rustfmt wouldn't be able to format that
    pub fn embed(&self) -> (TokenStream, Vec<(String, Vec<u8>)>) {
        let prefix = self.qualification_prefix(true);

        let mut file_inclusion_toks = quote!();
        let mut files_for_inclusion = BTreeMap::<String, Vec<u8>>::new();
        let mut embedded_frame_count = 0usize;
        let spec = self
            .spec()
            .traverse_data::<Ident, !>(|data| {
                let id = hex::encode_upper(format!(
                    "{},{},{}",
                    data.file_range().start,
                    data.file_range().end,
                    data.file
                ));
                let ident = format_ident!("CHUNK_{}", id);
                let fname = format!("chunk.{}.bin", id);
                files_for_inclusion.entry(fname.clone()).or_insert_with(|| {
                    file_inclusion_toks.extend(quote! {
                        const #ident: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/", #fname));
                    });
                    self.pack_fill(self.fill_map.get(data))
                });
                Ok(ident)
            })
            .into_ok()
            .traverse_embedded_frames::<Ident, !>(|fill| {
                let ident = format_ident!("FRAME_{}", embedded_frame_count);
                let fname = format!("frame.{}.bin", embedded_frame_count);
                file_inclusion_toks.extend(quote! {
                    const #ident: #prefix SelfContained<#prefix EmbeddedFrame> = #prefix SelfContained::new(
                        #prefix embed_frame!(4096, *include_bytes!(concat!(env!("OUT_DIR"), "/", #fname)))
                    );
                });
                files_for_inclusion.insert(fname, self.fill_map.get_frame(self.granule_size(), fill));
                embedded_frame_count += 1;
                Ok(ident)
            })
            .into_ok();

        let types_mod = self.types_mod();
        let name_type = self.name_type(true);
        let fill_type = self.fill_type(true);
        let embedded_frame_type = quote!(#prefix SelfContained<#prefix EmbeddedFrame>);

        let objects = spec.named_objects().map(|NamedObject { name, object }| {
            let name = self.name_value(object, name);
            let object = self.embed_object(object);
            quote! {
                NamedObject {
                    name: #name,
                    object: #object,
                }
            }
        });

        let irqs = to_tokens_via_debug(&spec.irqs);
        let root_objects = to_tokens_via_debug(&spec.root_objects);
        let untyped_covers = to_tokens_via_debug(&spec.untyped_covers);
        let asid_slots = to_tokens_via_debug(&spec.asid_slots);

        let toks = quote! {
            #[allow(unused_imports)]
            pub const SPEC: #types_mod::Spec<'static, #name_type, #fill_type, #embedded_frame_type> = {

                use #types_mod::*;

                #file_inclusion_toks

                const NAMED_OBJECTS: &[NamedObject<#name_type, #fill_type, #embedded_frame_type>] = &[#(#objects,)*];

                Spec {
                    objects: Indirect::from_borrowed(NAMED_OBJECTS),
                    irqs: Indirect::from_borrowed(#irqs.as_slice()),
                    root_objects: #root_objects,
                    untyped_covers: Indirect::from_borrowed(#untyped_covers.as_slice()),
                    asid_slots: Indirect::from_borrowed(#asid_slots.as_slice()),
                }
            };
        };

        (toks, files_for_inclusion.into_iter().collect())
    }
}
