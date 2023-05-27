#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use sel4_root_task::{debug_println, main, panicking};

static F1_DROPPED: AtomicBool = AtomicBool::new(false);

#[main(stack_size = 4096 * 64, heap_size = 4096 * 16)] // TODO decrease stack size
fn main(_: &sel4::BootInfo) -> ! {
    let _ = panicking::catch_unwind(|| {
        f1();
    });
    assert!(F1_DROPPED.load(Ordering::SeqCst));
    whether_alloc();
    debug_println!("TEST_PASS");

    sel4::BootInfo::init_thread_tcb().tcb_suspend().unwrap();
    unreachable!()
}

fn f1() {
    [()].iter().for_each(f1_helper);
}

fn f1_helper(_: &()) -> () {
    let _ = F1Drop;
    panic!("test");
}

struct F1Drop;

impl Drop for F1Drop {
    fn drop(&mut self) {
        debug_println!("F1Drop::drop()");
        F1_DROPPED.store(true, Ordering::SeqCst);
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        extern crate alloc;

        use alloc::borrow::ToOwned;
        use alloc::string::String;

        fn whether_alloc() {
            let r = panicking::catch_unwind(|| {
                panicking::panic_any::<String>("foo".to_owned());
            });
            assert_eq!(r.err().unwrap().inner().downcast_ref::<String>().unwrap().as_str(), "foo");
        }
    } else {
        use core::mem;

        use panicking::{FromPayloadValue, IntoPayloadValue, PayloadValue, TryFromPayload, PAYLOAD_VALUE_SIZE};

        fn whether_alloc() {
            let r = panicking::catch_unwind(|| {
                panicking::panic_any(Foo(1337));
            });
            assert!(matches!(Foo::try_from_payload(&r.err().unwrap()).unwrap(), Foo(1337)));
        }

        #[derive(Copy, Clone)]
        struct Foo(usize);

        unsafe impl IntoPayloadValue for Foo {
            fn into_payload_value(self) -> PayloadValue {
                let mut outer = [0; PAYLOAD_VALUE_SIZE];
                let inner = self.0.to_ne_bytes();
                outer[..inner.len()].copy_from_slice(&inner);
                outer
            }
        }

        unsafe impl FromPayloadValue for Foo {
            fn from_payload_value(payload_value: &PayloadValue) -> Self {
                Foo(usize::from_ne_bytes(payload_value[..mem::size_of::<usize>()].try_into().unwrap()))
            }
        }
    }
}
