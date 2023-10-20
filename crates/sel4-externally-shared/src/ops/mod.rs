pub use volatile::ops::*;

mod bytewise_ops;
mod normal_ops;
mod unordered_atomic_ops;
mod zerocopy_ops;

pub use bytewise_ops::BytewiseOps;
pub use normal_ops::NormalOps;
pub use unordered_atomic_ops::UnorderedAtomicOps;
pub use zerocopy_ops::ZerocopyOps;
