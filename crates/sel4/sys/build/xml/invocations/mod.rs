//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

#![allow(clippy::eq_op)]
#![allow(clippy::nonminimal_bool)]
#![feature(int_roundings)]
use std::collections::BTreeMap;
use std::fmt::Write;
use std::ops::Range;
use std::path::Path;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};

use sel4_config::sel4_cfg_bool;

use super::{parse_xml, Condition};

mod parse;
use parse::*;

const WORD_SIZE: usize = sel4_config::sel4_cfg_usize!(WORD_SIZE);

#[sel4_config::sel4_cfg(WORD_SIZE = "32")]
type Word = u32;

#[sel4_config::sel4_cfg(WORD_SIZE = "64")]
type Word = u64;

pub fn generate_rust(
    blocklist_for_bindgen: &mut Vec<String>,
    interface_xml_paths: &[impl AsRef<Path>],
) -> (TokenStream, TokenStream, TokenStream) {
    let mut structs = vec![];
    let mut interfaces = vec![];
    for f in interface_xml_paths {
        let api = Api::parse(&parse_xml(f));
        structs.extend(api.structs);
        interfaces.extend(api.interfaces);
    }

    let invocation_labels = {
        let mut acc = vec!["InvalidInvocation".to_owned()];
        for interface in interfaces.iter() {
            for method in interface.methods.iter() {
                if Condition::eval_option(&method.condition) {
                    acc.push(method.id.clone());
                }
            }
        }
        acc
    };

    let invocation_label_toks = {
        let mut toks = quote!();
        let ty = quote!(u32);
        for (i, id) in invocation_labels.iter().enumerate() {
            let ident = format_ident!("{}", id);
            let i = u32::try_from(i).unwrap();
            toks.extend(quote! {
                pub const #ident: #ty = #i;
            })
        }
        toks
    };

    let mut invocation_generator = InvocationGenerator::new(blocklist_for_bindgen, &structs);
    for interface in interfaces.iter() {
        for method in interface.methods.iter() {
            if Condition::eval_option(&method.condition) {
                let (in_params, out_params) = method.partition_parameters();
                invocation_generator.generate_invocation(
                    &interface.name,
                    &method.name,
                    &method.id,
                    &in_params,
                    &out_params,
                );
            }
        }
    }
    let (native_toks, wrapper_toks) = invocation_generator.generate_module();

    (invocation_label_toks, native_toks, wrapper_toks)
}

pub struct InvocationGenerator<'a> {
    parameter_types: ParameterTypes,
    blocklist_for_bindgen: &'a mut Vec<String>,
    ret_struct_toks: TokenStream,
    ipc_buffer_method_toks: TokenStream,
    wrapper_toks: TokenStream,
}

impl<'a> InvocationGenerator<'a> {
    pub fn new(blocklist_for_bindgen: &'a mut Vec<String>, structs: &[Struct]) -> Self {
        Self {
            parameter_types: ParameterTypes::create(structs),
            blocklist_for_bindgen,
            ret_struct_toks: quote!(),
            ipc_buffer_method_toks: quote!(),
            wrapper_toks: quote!(),
        }
    }

    pub fn generate_module(self) -> (TokenStream, TokenStream) {
        let ret_struct_toks = self.ret_struct_toks;
        let ipc_buffer_method_toks = self.ipc_buffer_method_toks;
        let wrapper_toks = self.wrapper_toks;

        let native_toks = quote! {
            #ret_struct_toks

            impl seL4_IPCBuffer {
                #ipc_buffer_method_toks
            }
        };

        (native_toks, wrapper_toks)
    }

    pub fn generate_invocation(
        &mut self,
        interface_name: &str,
        method_name: &str,
        invocation_id: &str,
        in_params: &[Parameter],
        out_params: &[Parameter],
    ) {
        let fn_name = format!("{interface_name}_{method_name}");
        let fn_ident = format_ident!("{}", fn_name);
        let interface_ident = format_ident!("{}", interface_name);
        let ret_struct_ident = format_ident!("{}_ret", fn_name);

        let param_list_generator =
            ParameterListGenerator::new(&self.parameter_types, in_params, out_params);
        let param_list_for_type_sig =
            param_list_generator.generate(ParameterListRole::NativeFunctionTypeSignature);
        let param_list_for_wrapper_type_sig =
            param_list_generator.generate(ParameterListRole::WrapperFunctionTypeSignature);
        let param_list_for_wrapper_expr =
            param_list_generator.generate(ParameterListRole::WrapperFunctionArguments);

        let out_params_passed_by_value = out_params
            .iter()
            .filter(|param| !self.parameter_types.get(&param.ty).pass_by_reference())
            .collect::<Vec<&Parameter>>();

        let use_ret_struct = !out_params_passed_by_value.is_empty();
        let ret_struct_definition = if use_ret_struct {
            self.generate_ret_struct_definition(&ret_struct_ident, &out_params_passed_by_value)
        } else {
            quote!()
        };
        let ret_type_path = if use_ret_struct {
            quote!(#ret_struct_ident)
        } else {
            quote!(seL4_Error::Type)
        };
        if use_ret_struct {
            // Add C name for this struct to blocklist
            self.blocklist_for_bindgen.push(fn_name)
        }

        let (marshalling_toks, num_msg_regs, num_caps) = self.generate_marshalling(in_params);
        let num_msg_regs = Word::try_from(num_msg_regs).unwrap();
        let num_caps = Word::try_from(num_caps).unwrap();

        let invocation_label_path = {
            let ident = format_ident!("{}", invocation_id);
            quote!(invocation_label::#ident)
        };

        let ret_struct_declaration = if use_ret_struct {
            quote! {
                let mut ret: #ret_struct_ident = unsafe { core::mem::zeroed() }; // TODO
                ret.error = err;
            }
        } else {
            quote!()
        };

        let unmarshalling_toks = self.generate_unmarshalling(out_params);

        let ret_expr = if use_ret_struct {
            quote!(ret)
        } else {
            quote!(err)
        };

        let trace_toks = {
            let fmt_string = format!(
                "{}_{}(_service={{:?}}{})",
                interface_name,
                method_name,
                in_params.iter().fold(String::new(), |mut f, param| {
                    write!(f, ", {}={{:?}}", param.name).unwrap();
                    f
                })
            );
            let fmt_args = in_params.iter().map(|param| raw_ident(&param.name));
            quote! {
                log::trace!(#fmt_string, service, #(#fmt_args,)*);
            }
        };

        self.ret_struct_toks.extend(quote! {
            #ret_struct_definition
        });

        self.ipc_buffer_method_toks.extend(quote! {
            pub fn #fn_ident(&mut self, service: #interface_ident, #(#param_list_for_type_sig,)*) -> #ret_type_path {
                #trace_toks
                #marshalling_toks
                let info_in = seL4_MessageInfo::new(#invocation_label_path.into(), 0, #num_caps, #num_msg_regs);
                let info_out = self.seL4_Call(service, info_in);
                let err: seL4_Error::Type = info_out.get_label().try_into().unwrap();
                #ret_struct_declaration
                #unmarshalling_toks
                #ret_expr
            }
        });

        self.wrapper_toks.extend(quote! {
            #[no_mangle]
            pub extern "C" fn #fn_ident(service: crate::#interface_ident, #(#param_list_for_wrapper_type_sig,)*) -> crate::#ret_type_path {
                get_ipc_buffer_mut().#fn_ident(service, #(#param_list_for_wrapper_expr,)*)
            }
        });
    }

    fn generate_ret_struct_definition(
        &self,
        ret_struct_ident: &Ident,
        out_params_passed_by_value: &[&Parameter],
    ) -> TokenStream {
        let ret_struct_fields = out_params_passed_by_value.iter().map(|param| {
            let name = format_ident!("{}", param.name);
            let ty = format_ident!("{}", param.ty);
            quote! {
                #name: #ty
            }
        });
        quote! {
            #[cfg_attr(feature = "wrappers", repr(C))] // TODO better to just be unconditionally repr(C) for the sake of consistency?
            pub struct #ret_struct_ident {
                pub error: seL4_Error::Type,
                #(pub #ret_struct_fields),*
            }
        }
    }

    fn generate_marshalling(&self, in_params: &[Parameter]) -> (TokenStream, usize, usize) {
        let mut toks = quote!();
        let mut layout_helper = LayoutHelper::new(&self.parameter_types);
        for param in in_params {
            let name = raw_ident(&param.name);
            match self.parameter_types.get(&param.ty) {
                ParameterType::Capability => {
                    let ix = layout_helper.lay_down_cap();
                    toks.extend(quote! {
                        self.set_cap(#ix, #name);
                    })
                }
                non_cap => {
                    let range = layout_helper.lay_down_data(&param.ty);
                    let start = range.start;
                    let end = range.end;
                    match non_cap {
                        ParameterType::Primitive { .. } => {
                            toks.extend(quote! {
                                self.set_mr_bits(#start..#end, #name);
                            });
                        }
                        ParameterType::Bitfield => toks.extend(quote! {
                            self.set_mr_bits_from_slice(#start..#end, #name.0.inner());
                        }),
                        ParameterType::Struct { members } => {
                            assert!(self.parameter_types.get(&param.ty).pass_by_reference());
                            let name = format_ident!("{}", param.name);
                            for (i, member) in members.iter().enumerate() {
                                let member = format_ident!("{}", member);
                                let member_start = start + i * WORD_SIZE;
                                let member_end = member_start + WORD_SIZE;
                                toks.extend(quote! {
                                    self.set_mr_bits(#member_start..#member_end, #name.#member);
                                })
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                }
            }
        }

        let num_msg_regs = layout_helper.num_msg_regs();
        let num_caps = layout_helper.num_caps();
        let toks = quote! {
            self.msg[..#num_msg_regs].fill(0);
            #toks
        };
        (toks, num_msg_regs, num_caps)
    }

    fn generate_unmarshalling(&self, out_params: &[Parameter]) -> TokenStream {
        let mut toks = quote!();
        let mut layout_helper = LayoutHelper::new(&self.parameter_types);
        for param in out_params {
            let range = layout_helper.lay_down_data(&param.ty);
            let start = range.start;
            let end = range.end;
            match self.parameter_types.get(&param.ty) {
                ParameterType::Primitive {
                    module_enum: false, ..
                } => {
                    let name = format_ident!("{}", param.name);
                    toks.extend(quote! {
                        ret.#name = self.get_mr_bits(#start..#end);
                    })
                }
                ParameterType::Struct { members } => {
                    assert!(self.parameter_types.get(&param.ty).pass_by_reference());
                    let name = format_ident!("{}", param.name);
                    for (i, member) in members.iter().enumerate() {
                        let member = format_ident!("{}", member);
                        let member_start = start + i * WORD_SIZE;
                        let member_end = member_start + WORD_SIZE;
                        toks.extend(quote! {
                            #name.#member = self.get_mr_bits(#member_start..#member_end);
                        })
                    }
                }
                _ => {
                    panic!()
                }
            }
        }
        toks
    }
}

struct ParameterListGenerator<'a> {
    parameter_types: &'a ParameterTypes,
    in_params: &'a [Parameter],
    out_params: &'a [Parameter],
}

#[derive(Clone, PartialEq, Eq)]
enum ParameterListRole {
    NativeFunctionTypeSignature,
    WrapperFunctionTypeSignature,
    WrapperFunctionArguments,
}

impl ParameterListRole {
    fn is_type_signature(&self) -> bool {
        *self != Self::WrapperFunctionArguments
    }

    fn is_native(&self) -> bool {
        *self == Self::NativeFunctionTypeSignature
    }
}

impl<'a> ParameterListGenerator<'a> {
    fn new(
        parameter_types: &'a ParameterTypes,
        in_params: &'a [Parameter],
        out_params: &'a [Parameter],
    ) -> Self {
        Self {
            parameter_types,
            in_params,
            out_params,
        }
    }

    // TODO use * instead of & for wrappers
    fn generate(&self, role: ParameterListRole) -> Vec<TokenStream> {
        let ty_path_prefix = if role.is_native() {
            quote!()
        } else {
            quote!(crate::)
        };
        let mut param_list = vec![];
        for param in self.in_params.iter().chain(self.out_params.iter()) {
            let var_ident = raw_ident(&param.name);
            let ty = format_ident!("{}", param.ty);
            let ty_suffix = match self.parameter_types.get(&param.ty) {
                ParameterType::Primitive {
                    module_enum: true, ..
                } => quote!(::Type),
                _ => quote!(),
            };
            match param.direction {
                ParameterDirection::In => {
                    let ty_modifier = if self.parameter_types.get(&param.ty).pass_by_reference() {
                        quote!(&)
                    } else {
                        quote!()
                    };
                    param_list.push((
                        quote!(#var_ident),
                        quote!(#ty_modifier #ty_path_prefix #ty #ty_suffix),
                    ));
                }
                ParameterDirection::Out => {
                    if self.parameter_types.get(&param.ty).pass_by_reference() {
                        param_list.push((
                            quote!(#var_ident),
                            quote!(&mut #ty_path_prefix #ty #ty_suffix),
                        ));
                    }
                }
            }
        }
        param_list
            .into_iter()
            .map(move |(var, ty)| {
                if role.is_type_signature() {
                    quote!(#var: #ty)
                } else {
                    quote!(#var)
                }
            })
            .collect()
    }
}

struct LayoutHelper<'a> {
    parameter_types: &'a ParameterTypes,
    data_cursor: usize,
    cap_cursor: usize,
}

impl<'a> LayoutHelper<'a> {
    fn new(parameter_types: &'a ParameterTypes) -> Self {
        Self {
            parameter_types,
            data_cursor: 0,
            cap_cursor: 0,
        }
    }

    fn lay_down_data(&mut self, parameter_type_name: &str) -> Range<usize> {
        let width = self.parameter_types.get(parameter_type_name).width();
        let start = self.data_cursor.next_multiple_of(width.min(WORD_SIZE));
        let end = start + width;
        self.data_cursor = end;
        start..end
    }

    fn lay_down_cap(&mut self) -> usize {
        let ix = self.cap_cursor;
        self.cap_cursor += 1;
        ix
    }

    fn num_msg_regs(&self) -> usize {
        self.data_cursor.next_multiple_of(WORD_SIZE) / WORD_SIZE
    }

    fn num_caps(&self) -> usize {
        self.cap_cursor
    }
}

type ParameterTypeName = String;

struct ParameterTypes {
    inner: BTreeMap<ParameterTypeName, ParameterType>,
}

enum ParameterType {
    Primitive { width: usize, module_enum: bool },
    Capability,
    Bitfield,
    Struct { members: Vec<String> },
}

impl ParameterType {
    fn width(&self) -> usize {
        match self {
            Self::Primitive { width, .. } => *width,
            Self::Capability => WORD_SIZE,
            Self::Bitfield => WORD_SIZE,
            Self::Struct { members } => WORD_SIZE * members.len(),
        }
    }

    fn pass_by_reference(&self) -> bool {
        match self {
            Self::Struct { members } => members.len() > 1,
            _ => false,
        }
    }
}

impl ParameterTypes {
    fn empty() -> Self {
        Self {
            inner: BTreeMap::new(),
        }
    }

    fn get(&self, name: &str) -> &ParameterType {
        self.inner.get(name).unwrap()
    }

    fn insert_primitive(&mut self, name: impl ToString, width: usize) {
        self.insert_primitive_inner(name, width, false);
    }

    fn insert_enum(&mut self, name: impl ToString, width: usize) {
        self.insert_primitive_inner(name, width, true);
    }

    fn insert_primitive_inner(&mut self, name: impl ToString, width: usize, module_enum: bool) {
        self.inner.insert(
            name.to_string(),
            ParameterType::Primitive { width, module_enum },
        );
    }

    fn insert_capability(&mut self, name: impl ToString) {
        self.inner
            .insert(name.to_string(), ParameterType::Capability);
    }

    fn insert_bitfield(&mut self, name: impl ToString) {
        self.inner.insert(name.to_string(), ParameterType::Bitfield);
    }

    fn insert_struct(&mut self, name: impl ToString, members: &[String]) {
        self.inner.insert(
            name.to_string(),
            ParameterType::Struct {
                members: members.to_vec(),
            },
        );
    }

    fn create(structs: &[Struct]) -> Self {
        let mut this = Self::empty();

        this.insert_primitive("seL4_Uint8", 8);
        this.insert_primitive("seL4_Uint16", 16);
        this.insert_primitive("seL4_Uint32", 32);
        this.insert_primitive("seL4_Uint64", 64);
        this.insert_primitive("seL4_Time", 64);
        this.insert_primitive("seL4_Word", WORD_SIZE);
        this.insert_primitive("seL4_Bool", 1);

        this.insert_bitfield("seL4_CapRights_t");

        this.insert_capability("seL4_CPtr");
        this.insert_capability("seL4_CNode");
        this.insert_capability("seL4_IRQHandler");
        this.insert_capability("seL4_IRQControl");
        this.insert_capability("seL4_TCB");
        this.insert_capability("seL4_Untyped");
        this.insert_capability("seL4_DomainSet");
        this.insert_capability("seL4_SchedContext");
        this.insert_capability("seL4_SchedControl");

        if sel4_cfg_bool!(ARCH_AARCH64) | sel4_cfg_bool!(ARCH_AARCH32) {
            this.insert_enum("seL4_ARM_VMAttributes", WORD_SIZE);
            this.insert_capability("seL4_ARM_Page");
            this.insert_capability("seL4_ARM_PageTable");
            this.insert_capability("seL4_ARM_PageDirectory");
            if sel4_cfg_bool!(ARCH_AARCH64) {
                this.insert_capability("seL4_ARM_PageUpperDirectory");
                this.insert_capability("seL4_ARM_PageGlobalDirectory");
                this.insert_capability("seL4_ARM_VSpace");
            }
            this.insert_capability("seL4_ARM_ASIDControl");
            this.insert_capability("seL4_ARM_ASIDPool");
            this.insert_capability("seL4_ARM_VCPU");
            this.insert_capability("seL4_ARM_IOSpace");
            this.insert_capability("seL4_ARM_IOPageTable");
        }

        if sel4_cfg_bool!(ARCH_RISCV64) || sel4_cfg_bool!(ARCH_RISCV32) {
            this.insert_enum("seL4_RISCV_VMAttributes", WORD_SIZE);
            this.insert_capability("seL4_RISCV_Page");
            this.insert_capability("seL4_RISCV_PageTable");
            this.insert_capability("seL4_RISCV_ASIDControl");
            this.insert_capability("seL4_RISCV_ASIDPool");
        }

        if sel4_cfg_bool!(ARCH_X86_64) {
            this.insert_enum("seL4_X86_VMAttributes", WORD_SIZE);
            this.insert_capability("seL4_X86_IOPort");
            this.insert_capability("seL4_X86_IOPortControl");
            this.insert_capability("seL4_X86_ASIDControl");
            this.insert_capability("seL4_X86_ASIDPool");
            this.insert_capability("seL4_X86_IOSpace");
            this.insert_capability("seL4_X86_Page");
            this.insert_capability("seL4_X64_PML4");
            this.insert_capability("seL4_X86_PDPT");
            this.insert_capability("seL4_X86_PageDirectory");
            this.insert_capability("seL4_X86_PageTable");
            this.insert_capability("seL4_X86_IOPageTable");
            this.insert_capability("seL4_X86_VCPU");
            this.insert_capability("seL4_X86_EPTPML4");
            this.insert_capability("seL4_X86_EPTPDPT");
            this.insert_capability("seL4_X86_EPTPD");
            this.insert_capability("seL4_X86_EPTPT");
        }

        for struct_ in structs {
            this.insert_struct(&struct_.name, &struct_.members);
        }

        this
    }
}

fn raw_ident(s: &str) -> Ident {
    format_ident!("r#{}", s)
}
