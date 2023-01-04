mod whether_alloc;

pub(crate) use whether_alloc::*;

const RUST_EXCEPTION_CLASS: u64 = u64::from_be_bytes(*b"MOZ\0RUST");
