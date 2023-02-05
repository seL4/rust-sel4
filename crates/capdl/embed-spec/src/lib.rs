#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::collections::BTreeMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use capdl_types::*;

type SpecInput<'a> = Spec<'a, String, FillEntryContentFileAndBytes>;
type FillEntryContentFileAndBytes = (FillEntryContentFile, FillEntryContentBytes<'static>);

pub fn embed<'a>(
    spec: &'a SpecInput<'a>,
    include_names: bool,
    deflate_fill: bool,
) -> (TokenStream, Vec<(String, Vec<u8>)>) {
    let state = State::new(spec, include_names, deflate_fill);
    state.embed()
}

struct State<'a> {
    spec: &'a SpecInput<'a>,
    include_names: bool,
    deflate_fill: bool,
}

type FillEntryContentId = String;

impl<'a> State<'a> {
    fn new(spec: &'a SpecInput<'a>, include_names: bool, deflate_fill: bool) -> Self {
        Self {
            spec,
            include_names,
            deflate_fill,
        }
    }

    fn types_mod(&self) -> TokenStream {
        quote! { capdl_types }
    }

    fn embed_cap(&self, cap: &Cap) -> TokenStream {
        // code-saving hack
        let types_mod = self.types_mod();
        let debug = format!("{:?}", cap).parse::<TokenStream>().unwrap();
        quote! {
            {
                use #types_mod::cap::*;
                Cap::#debug
            }
        }
    }

    fn embed_cap_table(&self, slots: &[CapTableEntry]) -> TokenStream {
        let mut toks = quote!();
        for (i, cap) in slots {
            let cap = self.embed_cap(cap);
            toks.extend(quote! {
                (#i, #cap),
            })
        }
        quote! {
            Indirect::from_borrowed([#toks].as_slice())
        }
    }

    // TODO(nspin) support FillEntryContent::BootInfo
    fn embed_fill(&self, fill: &[FillEntry<FillEntryContentFileAndBytes>]) -> TokenStream {
        let mut toks = quote!();
        for entry in fill.iter() {
            let range = format!("{:?}", entry.range).parse::<TokenStream>().unwrap();
            let content_value = self.embedded_file_ident_from_id(&self.encode_fill_entry_to_id(
                entry.range.end - entry.range.start,
                entry.content.as_data().unwrap(),
            ));
            let (content_type, content_field) = if self.deflate_fill {
                (
                    quote!(FillEntryContentDeflatedBytes),
                    quote!(deflated_bytes),
                )
            } else {
                (quote!(FillEntryContentBytes), quote!(bytes))
            };
            toks.extend(quote! {
                FillEntry {
                    range: #range,
                    content: FillEntryContent::Data(#content_type {
                        #content_field: #content_value,
                    }),
                },
            })
        }
        quote! {
            Indirect::from_borrowed([#toks].as_slice())
        }
    }

    fn embed_object(&self, obj: &Object<FillEntryContentFileAndBytes>) -> TokenStream {
        match obj {
            Object::CNode(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                let size_bits = obj.size_bits;
                quote! {
                    Object::CNode(object::CNode {
                        size_bits: #size_bits,
                        slots: #toks,
                    })
                }
            }
            Object::TCB(obj) => {
                let types_mod = self.types_mod();
                let slots = self.embed_cap_table(&obj.slots);
                let extra = format!("{:?}", obj.extra).parse::<TokenStream>().unwrap();
                quote! {
                    {
                        use #types_mod::object::TCBExtraInfo;
                        Object::TCB(object::TCB {
                            slots: #slots,
                            extra: Indirect::from_borrowed(&#extra),
                        })
                    }
                }
            }
            Object::IRQ(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                quote! {
                    Object::IRQ(object::IRQ {
                        slots: #toks,
                    })
                }
            }
            Object::SmallPage(obj) => {
                let toks = self.embed_fill(&obj.fill);
                let paddr = format!("{:?}", obj.paddr).parse::<TokenStream>().unwrap();
                quote! {
                    Object::SmallPage(object::SmallPage {
                        paddr: #paddr,
                        fill: #toks,
                    })
                }
            }
            Object::LargePage(obj) => {
                let toks = self.embed_fill(&obj.fill);
                let paddr = format!("{:?}", obj.paddr).parse::<TokenStream>().unwrap();
                quote! {
                    Object::LargePage(object::LargePage {
                        paddr: #paddr,
                        fill: #toks,
                    })
                }
            }
            Object::PT(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                quote! {
                    Object::PT(object::PT {
                        slots: #toks,
                    })
                }
            }
            Object::PD(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                quote! {
                    Object::PD(object::PD {
                        slots: #toks,
                    })
                }
            }
            Object::PUD(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                quote! {
                    Object::PUD(object::PUD {
                        slots: #toks,
                    })
                }
            }
            Object::PGD(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                quote! {
                    Object::PGD(object::PGD {
                        slots: #toks,
                    })
                }
            }
            Object::ASIDPool(obj) => {
                let high = obj.high;
                quote! {
                    Object::ASIDPool(object::ASIDPool {
                        high: #high,
                    })
                }
            }
            Object::ArmIRQ(obj) => {
                let toks = self.embed_cap_table(&obj.slots);
                let extra = format!("{:?}", obj.extra).parse::<TokenStream>().unwrap();
                quote! {
                    Object::ArmIRQ(object::ArmIRQ {
                        slots: #toks,
                        extra: #extra,
                    })
                }
            }
            _ => {
                // code-saving hack
                let types_mod = self.types_mod();
                let debug = format!("{:?}", obj).parse::<TokenStream>().unwrap();
                quote! {
                    {
                        #[allow(unused_imports)]
                        use #types_mod::object::*;
                        Object::#debug
                    }
                }
            }
        }
    }

    fn embed_objects(&self) -> TokenStream {
        let mut toks = quote!();
        for NamedObject { name, object } in self.spec.named_objects() {
            let object = self.embed_object(object);
            let name = if self.include_names {
                let name = name.to_owned();
                quote! { #name }
            } else {
                quote! { Unnamed }
            };
            toks.extend(quote! {
                NamedObject {
                    name: #name,
                    object: #object,
                },
            })
        }
        // TODO(nspin)
        // This const declaration + type signature are just to appease the borrow checker.
        // There must be a more elegant way.
        let name_type = if self.include_names {
            quote!(&str)
        } else {
            quote!(Unnamed)
        };
        let fill_type = if self.deflate_fill {
            quote!(FillEntryContentDeflatedBytes)
        } else {
            quote!(FillEntryContentBytes)
        };
        quote! {
            Indirect::from_borrowed({
                const NAMED_OBJECTS: &[NamedObject<#name_type, #fill_type>] = &[#toks];
                NAMED_OBJECTS
            })
        }
    }

    fn embed_irqs(&self) -> TokenStream {
        let toks = format!("{:?}", self.spec.irqs)
            .parse::<TokenStream>()
            .unwrap();
        quote! {
            Indirect::from_borrowed(#toks.as_slice())
        }
    }

    fn embed_asid_slots(&self) -> TokenStream {
        let toks = format!("{:?}", self.spec.asid_slots)
            .parse::<TokenStream>()
            .unwrap();
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
        format!("{}.bin", id)
    }

    // NOTE
    // I would prefer this to return just the rhs, but rustfmt wouldn't be able to format that
    fn embed(&self) -> (TokenStream, Vec<(String, Vec<u8>)>) {
        let types_mod = self.types_mod();

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
                    let content = (if self.deflate_fill {
                        FillEntryContentDeflatedBytes::pack
                    } else {
                        FillEntryContentBytes::pack
                    })(&content_bytes.bytes);
                    fills.insert(id, content);
                }
                Ok::<(), !>(())
            })
            .into_ok();

        let name_type = if self.include_names {
            quote! { &'static str }
        } else {
            quote! { #types_mod::Unnamed }
        };
        let objects = self.embed_objects();
        let irqs = self.embed_irqs();
        let asid_slots = self.embed_asid_slots();
        let fill_type = if self.deflate_fill {
            quote!(#types_mod::FillEntryContentDeflatedBytes)
        } else {
            quote!(#types_mod::FillEntryContentBytes)
        };
        let toks = quote! {
            pub const SPEC: #types_mod::Spec<'static, #name_type, #fill_type> = {
                use #types_mod::*;

                #file_definition_toks

                Spec {
                    objects: #objects,
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
