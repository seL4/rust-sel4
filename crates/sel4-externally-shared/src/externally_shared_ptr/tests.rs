use crate::{
    access::{ReadOnly, ReadWrite, WriteOnly},
    map_field, ExternallySharedPtr,
};
use core::ptr::NonNull;

#[test]
fn test_read() {
    let val = 42;
    assert_eq!(
        unsafe { ExternallySharedPtr::new_read_only(NonNull::from(&val)) }.read(),
        42
    );
}

#[test]
fn test_write() {
    let mut val = 50;
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(&mut val)) };
    shared.write(50);
    assert_eq!(val, 50);
}

#[test]
fn test_update() {
    let mut val = 42;
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(&mut val)) };
    shared.update(|v| v + 1);
    assert_eq!(val, 43);
}

#[test]
fn test_access() {
    let mut val: i64 = 42;

    // ReadWrite
    assert_eq!(
        unsafe { ExternallySharedPtr::new_restricted(ReadWrite, NonNull::from(&mut val)) }.read(),
        42
    );
    unsafe { ExternallySharedPtr::new_restricted(ReadWrite, NonNull::from(&mut val)) }.write(50);
    assert_eq!(val, 50);
    unsafe { ExternallySharedPtr::new_restricted(ReadWrite, NonNull::from(&mut val)) }
        .update(|i| i + 1);
    assert_eq!(val, 51);

    // ReadOnly and WriteOnly
    assert_eq!(
        unsafe { ExternallySharedPtr::new_restricted(ReadOnly, NonNull::from(&mut val)) }.read(),
        51
    );
    unsafe { ExternallySharedPtr::new_restricted(WriteOnly, NonNull::from(&mut val)) }.write(12);
    assert_eq!(val, 12);
}

#[test]
fn test_struct() {
    #[derive(Debug, PartialEq)]
    struct S {
        field_1: u32,
        field_2: bool,
    }

    let mut val = S {
        field_1: 60,
        field_2: true,
    };
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(&mut val)) };
    unsafe {
        shared.map(|s| NonNull::new(core::ptr::addr_of_mut!((*s.as_ptr()).field_1)).unwrap())
    }
    .update(|v| v + 1);
    let field_2 = unsafe {
        shared.map(|s| NonNull::new(core::ptr::addr_of_mut!((*s.as_ptr()).field_2)).unwrap())
    };
    assert!(field_2.read());
    field_2.write(false);
    assert_eq!(
        val,
        S {
            field_1: 61,
            field_2: false
        }
    );
}

#[test]
fn test_struct_macro() {
    #[derive(Debug, PartialEq)]
    struct S {
        field_1: u32,
        field_2: bool,
    }

    let mut val = S {
        field_1: 60,
        field_2: true,
    };
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(&mut val)) };
    let field_1 = map_field!(shared.field_1);
    field_1.update(|v| v + 1);
    let field_2 = map_field!(shared.field_2);
    assert!(field_2.read());
    field_2.write(false);
    assert_eq!(
        val,
        S {
            field_1: 61,
            field_2: false
        }
    );
}

#[cfg(feature = "unstable")]
#[test]
fn test_slice() {
    let val: &mut [u32] = &mut [1, 2, 3];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    shared.index(0).update(|v| v + 1);

    let mut dst = [0; 3];
    shared.copy_into_slice(&mut dst);
    assert_eq!(dst, [2, 2, 3]);
}

#[cfg(feature = "unstable")]
#[test]
#[should_panic]
fn test_bounds_check_1() {
    let val: &mut [u32] = &mut [1, 2, 3];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    shared.index(3);
}

#[cfg(feature = "unstable")]
#[test]
#[should_panic]
fn test_bounds_check_2() {
    let val: &mut [u32] = &mut [1, 2, 3];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    #[allow(clippy::reversed_empty_ranges)]
    shared.index(2..1);
}

#[cfg(feature = "unstable")]
#[test]
#[should_panic]
fn test_bounds_check_3() {
    let val: &mut [u32] = &mut [1, 2, 3];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    shared.index(4..); // `3..` is is still ok (see next test)
}

#[cfg(feature = "unstable")]
#[test]
fn test_bounds_check_4() {
    let val: &mut [u32] = &mut [1, 2, 3];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    assert_eq!(shared.index(3..).len(), 0);
}

#[cfg(feature = "unstable")]
#[test]
#[should_panic]
fn test_bounds_check_5() {
    let val: &mut [u32] = &mut [1, 2, 3];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    shared.index(..4);
}

#[cfg(feature = "unstable")]
#[test]
fn test_chunks() {
    let val: &mut [u32] = &mut [1, 2, 3, 4, 5, 6];
    let shared = unsafe { ExternallySharedPtr::new(NonNull::from(val)) };
    let chunks = shared.as_chunks().0;
    chunks.index(1).write([10, 11, 12]);
    assert_eq!(chunks.index(0).read(), [1, 2, 3]);
    assert_eq!(chunks.index(1).read(), [10, 11, 12]);
}
