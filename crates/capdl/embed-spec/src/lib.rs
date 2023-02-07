#![feature(let_chains)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::collections::BTreeMap;
use std::fmt;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens};

use capdl_types::*;

// TODO(nspin)
// In a few cases, we use a local const declaration to appease the borrow checker.
// Using an exposed constructor of `Indirect` would be one way around this.

type InputSpec<'a> = Spec<'a, String, FillEntryContentFileAndBytes>;
type FillEntryContentFileAndBytes = (FillEntryContentFile, FillEntryContentBytes<'static>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub object_names_level: ObjectNamesLevel,
    pub deflate_fill: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectNamesLevel {
    All,
    JustTCBs,
    None,
}

impl Config {
    pub fn embed<'a>(&'a self, spec: &'a InputSpec<'a>) -> (TokenStream, Vec<(String, Vec<u8>)>) {
        Embedding { config: self, spec }.embed()
    }
}

struct Embedding<'a> {
    config: &'a Config,
    spec: &'a InputSpec<'a>,
}

type FillEntryContentId = String;

fn to_tokens_via_debug(value: impl fmt::Debug) -> TokenStream {
    format!("{:?}", value).parse::<TokenStream>().unwrap()
}

impl<'a> Embedding<'a> {
    fn types_mod(&self) -> TokenStream {
        quote! { capdl_types }
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

    fn embed_fill(&self, fill: &[FillEntry<FillEntryContentFileAndBytes>]) -> TokenStream {
        let entries = fill.iter().map(|entry| {
            let range = to_tokens_via_debug(&entry.range);
            let content = match &entry.content {
                FillEntryContent::Data(content_data) => {
                    let inner_value = self.embedded_file_ident_from_id(
                        &self.encode_fill_entry_to_id(entry.range.len(), &content_data),
                    );
                    let outer_value = self.fill_value(inner_value);
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
            {
                use FillEntryContent::*;
                use FillEntryContentBootInfoId::*;
                Indirect::from_borrowed([#(#entries,)*].as_slice())
            }
        }
    }

    fn patch_field(&self, expr_struct: &mut syn::ExprStruct, field_name: &str, value: syn::Expr) {
        for field in expr_struct.fields.iter_mut() {
            if let syn::Member::Named(ident) = &field.member && ident == field_name {
                field.expr = value.clone();
            }
        }
    }

    fn embed_object_with_cap_table(&self, obj: &(impl fmt::Debug + HasCapTable)) -> TokenStream {
        let mut expr_struct = syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(&obj)).unwrap();
        self.patch_field(
            &mut expr_struct,
            "slots",
            syn::parse2::<syn::Expr>(self.embed_cap_table(&obj.slots())).unwrap(),
        );
        expr_struct.to_token_stream()
    }

    fn embed_object_with_fill(
        &self,
        obj: impl fmt::Debug,
        fill: &[FillEntry<FillEntryContentFileAndBytes>],
    ) -> TokenStream {
        let mut expr_struct = syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(&obj)).unwrap();
        self.patch_field(
            &mut expr_struct,
            "fill",
            syn::parse2::<syn::Expr>(self.embed_fill(fill)).unwrap(),
        );
        expr_struct.to_token_stream()
    }

    fn embed_object(&self, obj: &Object<FillEntryContentFileAndBytes>) -> TokenStream {
        match obj {
            Object::CNode(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::CNode(object::#toks))
            }
            Object::TCB(obj) => {
                let mut expr_struct =
                    syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(&obj)).unwrap();
                self.patch_field(
                    &mut expr_struct,
                    "slots",
                    syn::parse2::<syn::Expr>(self.embed_cap_table(&obj.slots())).unwrap(),
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
                let toks = self.embed_object_with_fill(obj, &obj.fill);
                quote!(Object::Frame(object::#toks))
            }
            Object::PT(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::PT(object::#toks))
            }
            Object::PD(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::PD(object::#toks))
            }
            Object::PUD(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::PUD(object::#toks))
            }
            Object::PGD(obj) => {
                let toks = self.embed_object_with_cap_table(obj);
                quote!(Object::PGD(object::#toks))
            }
            Object::ArmIRQ(obj) => {
                let mut expr_struct =
                    syn::parse2::<syn::ExprStruct>(to_tokens_via_debug(&obj)).unwrap();
                self.patch_field(
                    &mut expr_struct,
                    "slots",
                    syn::parse2::<syn::Expr>(self.embed_cap_table(&obj.slots())).unwrap(),
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
        match self.config.object_names_level {
            ObjectNamesLevel::All => quote!(&str),
            ObjectNamesLevel::JustTCBs => quote!(Option<&str>),
            ObjectNamesLevel::None => quote!(#prefix Unnamed),
        }
    }

    fn name_value<F>(&self, obj: &Object<'a, F>, name: &str) -> TokenStream {
        match self.config.object_names_level {
            ObjectNamesLevel::All => quote!(#name),
            ObjectNamesLevel::JustTCBs => match obj {
                Object::TCB(_) => quote!(Some(#name)),
                _ => quote!(None),
            },
            ObjectNamesLevel::None => quote!(Unnamed),
        }
    }

    fn fill_type(&self, qualify: bool) -> TokenStream {
        let prefix = self.qualification_prefix(qualify);
        let ty = if self.config.deflate_fill {
            quote!(FillEntryContentDeflatedBytes)
        } else {
            quote!(FillEntryContentBytes)
        };
        quote!(#prefix #ty)
    }

    fn fill_value(&self, field_value: impl ToTokens) -> TokenStream {
        let constructor = self.fill_type(false);
        let field_name = if self.config.deflate_fill {
            quote!(deflated_bytes)
        } else {
            quote!(bytes)
        };
        quote! {
            #constructor {
                #field_name: #field_value
            }
        }
    }

    fn pack_fill(&self, bytes: &[u8]) -> Vec<u8> {
        (if self.config.deflate_fill {
            FillEntryContentDeflatedBytes::pack
        } else {
            FillEntryContentBytes::pack
        })(bytes)
    }

    fn embed_objects(&self) -> TokenStream {
        let toks = self
            .spec
            .named_objects()
            .map(|NamedObject { name, object }| {
                let name = self.name_value(object, name);
                let object = self.embed_object(object);
                quote! {
                    NamedObject {
                        name: #name,
                        object: #object,
                    }
                }
            });
        quote! {
            [#(#toks,)*]
        }
    }

    fn embed_irqs(&self) -> TokenStream {
        let toks = to_tokens_via_debug(&self.spec.irqs);
        quote! {
            Indirect::from_borrowed(#toks.as_slice())
        }
    }

    fn embed_asid_slots(&self) -> TokenStream {
        let toks = to_tokens_via_debug(&self.spec.asid_slots);
        quote! {
            Indirect::from_borrowed(#toks.as_slice())
        }
    }

    fn encode_fill_entry_to_id(
        &self,
        length: usize,
        fill_entry: &FillEntryContentFileAndBytes,
    ) -> FillEntryContentId {
        let (content_file, _) = &fill_entry;
        hex::encode(format!(
            "{},{},{}",
            content_file.file_offset, length, content_file.file
        ))
    }

    fn embedded_file_ident_from_id(&self, id: &FillEntryContentId) -> Ident {
        format_ident!("embedded_file_{}", id)
    }

    fn embedded_file_fname_from_id(&self, id: &FillEntryContentId) -> String {
        format!("fragment.{}.bin", id)
    }

    // NOTE
    // I would prefer this to return just the rhs, but rustfmt wouldn't be able to format that
    fn embed(&self) -> (TokenStream, Vec<(String, Vec<u8>)>) {
        let mut file_definition_toks = quote!();
        let mut fills = BTreeMap::<FillEntryContentId, Vec<u8>>::new();
        self.spec
            .traverse_fill_with_context(|length, content| {
                let (_, content_bytes) = content;
                let id = self.encode_fill_entry_to_id(length, content);
                if !fills.contains_key(&id) {
                    let ident = self.embedded_file_ident_from_id(&id);
                    let fname = self.embedded_file_fname_from_id(&id);
                    file_definition_toks.extend(quote! {
                        #[allow(non_upper_case_globals)]
                        const #ident: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/", #fname));
                    });
                    let content = self.pack_fill(&content_bytes.bytes);
                    fills.insert(id, content);
                }
                Ok::<(), !>(())
            })
            .into_ok();

        let types_mod = self.types_mod();
        let name_type = self.name_type(true);
        let fill_type = self.fill_type(true);

        let objects = self.embed_objects();
        let irqs = self.embed_irqs();
        let asid_slots = self.embed_asid_slots();

        let toks = quote! {
            #[allow(unused_imports)]
            pub const SPEC: #types_mod::Spec<'static, #name_type, #fill_type> = {

                use #types_mod::*;

                #file_definition_toks

                const NAMED_OBJECTS: &[NamedObject<#name_type, #fill_type>] = &#objects;

                Spec {
                    objects: Indirect::from_borrowed(NAMED_OBJECTS),
                    irqs: #irqs,
                    asid_slots: #asid_slots,
                }
            };
        };

        let fills = fills
            .into_iter()
            .map(|(id, bytes)| (self.embedded_file_fname_from_id(&id), bytes))
            .collect();
        (toks, fills)
    }
}
