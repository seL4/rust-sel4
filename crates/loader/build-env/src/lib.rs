use sel4_build_env::{PathVarType, SimpleVar, Var, SEL4_PREFIX_ENV};

pub const SEL4_KERNEL: Var<PathVarType<'static>> =
    Var::new("SEL4_KERNEL", SEL4_PREFIX_ENV, "boot/kernel.elf");
pub const SEL4_DTB: Var<PathVarType<'static>> =
    Var::new("SEL4_DTB", SEL4_PREFIX_ENV, "boot/kernel.dtb");

pub const SEL4_LOADER_CONFIG: SimpleVar<PathVarType<'static>> =
    SimpleVar::new("SEL4_LOADER_CONFIG");
pub const SEL4_APP: SimpleVar<PathVarType<'static>> = SimpleVar::new("SEL4_APP");
