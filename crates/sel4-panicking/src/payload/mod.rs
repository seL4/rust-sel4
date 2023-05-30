use core::mem;
use core::slice;

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        mod with_alloc;
        use with_alloc as whether_alloc;
    } else {
        mod without_alloc;
        use without_alloc as whether_alloc;
    }
}

pub use whether_alloc::*;

pub trait UpcastIntoPayload {
    fn upcast_into_payload(self) -> Payload;
}

#[derive(Clone, Copy)]
pub struct SmallPayloadValue([u8; Self::SIZE]);

impl SmallPayloadValue {
    pub const SIZE: usize = 32;

    pub const fn ensure_fits<T: FitsWithinSmallPayload>() {
        assert!(mem::size_of::<T>() <= Self::SIZE);
    }

    pub fn write<T: FitsWithinSmallPayload + Copy>(val: &T) -> Self {
        Self::ensure_fits::<T>();
        let val_bytes =
            unsafe { slice::from_raw_parts(val as *const T as *const u8, mem::size_of::<T>()) };
        let mut payload_arr = [0; Self::SIZE];
        payload_arr[..val_bytes.len()].copy_from_slice(val_bytes);
        Self(payload_arr)
    }

    pub fn read<T: FitsWithinSmallPayload + Copy>(&self) -> T {
        Self::ensure_fits::<T>();
        unsafe { mem::transmute_copy(&self.0) }
    }
}

pub trait FitsWithinSmallPayload {}

#[derive(Clone, Copy)]
pub(crate) struct NoPayload;

impl FitsWithinSmallPayload for NoPayload {}
