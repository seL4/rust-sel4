use core::arch::global_asm;

use aligned::{Aligned, A16};
use spin::{Barrier, RwLock};

use loader_payload_types::PayloadInfo;

use crate::{psci, secondary_core_main, NUM_SECONDARY_CORES};

static SECONDARY_CORE_INIT_INFO: RwLock<Option<SecondaryCoreInitInfo>> = RwLock::new(None);

struct SecondaryCoreInitInfo {
    core_id: usize,
    payload_info: PayloadInfo,
    barrier: Barrier,
}

pub(crate) fn start_secondary_cores(payload_info: &PayloadInfo) {
    for i in 0..NUM_SECONDARY_CORES {
        let core_id = i + 1;
        let start = (secondary_core_start as *const SecondaryCoreStartFn).to_bits();
        let sp = get_secondary_core_initial_stack_pointer(i);
        {
            let mut init_info = SECONDARY_CORE_INIT_INFO.write();
            *init_info = Some(SecondaryCoreInitInfo {
                core_id,
                payload_info: payload_info.clone(),
                barrier: Barrier::new(2),
            });
        }
        psci::cpu_on(
            core_id.try_into().unwrap(),
            start.try_into().unwrap(),
            sp.try_into().unwrap(),
        )
        .unwrap();
        {
            let init_info = SECONDARY_CORE_INIT_INFO.read();
            let init_info = init_info.as_ref().unwrap();
            init_info.barrier.wait();
        }
        log::debug!("Primary core: core {} up", core_id);
    }
}

type SecondaryCoreStartFn = extern "C" fn() -> !;

extern "C" {
    fn secondary_core_start() -> !;
}

global_asm! {
    r#"
        .global secondary_core_start
        .extern secondary_core_start_rust

        .section .text

        secondary_core_start:
            mov sp, x0
            b secondary_core_start_rust
    "#
}

#[no_mangle]
extern "C" fn secondary_core_start_rust() -> ! {
    let core_id;
    let payload_info;
    {
        let init_info = SECONDARY_CORE_INIT_INFO.read();
        let init_info = init_info.as_ref().unwrap();
        init_info.barrier.wait();
        core_id = init_info.core_id;
        payload_info = init_info.payload_info.clone();
    }
    log::debug!("Core {}: up", core_id);
    secondary_core_main(core_id, &payload_info)
}

//

const SECONDARY_STACK_SIZE: usize = 4096 * 2;
const SECONDARY_STACKS_SIZE: usize = SECONDARY_STACK_SIZE * NUM_SECONDARY_CORES;

static SECONDARY_STACKS: Aligned<A16, [u8; SECONDARY_STACKS_SIZE]> =
    Aligned([0; SECONDARY_STACKS_SIZE]);

fn get_secondary_core_initial_stack_pointer(i: usize) -> usize {
    unsafe {
        SECONDARY_STACKS
            .as_slice()
            .as_ptr()
            .offset(((i + 1) * SECONDARY_STACK_SIZE).try_into().unwrap())
            .to_bits()
    }
}
