use aligned::{Aligned, A16};
use spin::{Barrier, RwLock};

use loader_payload_types::PayloadInfo;

use crate::{plat, secondary_core_main, NUM_SECONDARY_CORES};

static SECONDARY_CORE_INIT_INFO: RwLock<Option<SecondaryCoreInitInfo>> = RwLock::new(None);

struct SecondaryCoreInitInfo {
    core_id: usize,
    payload_info: PayloadInfo,
    barrier: Barrier,
}

pub(crate) fn start_secondary_cores(payload_info: &PayloadInfo) {
    for i in 0..NUM_SECONDARY_CORES {
        let core_id = i + 1;
        let sp = get_secondary_core_initial_stack_pointer(i);
        {
            let mut init_info = SECONDARY_CORE_INIT_INFO.write();
            *init_info = Some(SecondaryCoreInitInfo {
                core_id,
                payload_info: payload_info.clone(),
                barrier: Barrier::new(2),
            });
        }
        log::debug!("Primary core: starting core {}", core_id);
        plat::smp::start_secondary_core(core_id, sp);
        {
            let init_info = SECONDARY_CORE_INIT_INFO.read();
            let init_info = init_info.as_ref().unwrap();
            init_info.barrier.wait();
        }
        log::debug!("Primary core: core {} up", core_id);
    }
}

#[no_mangle]
extern "C" fn secondary_core_entry() -> ! {
    // crate::fmt::debug_println_without_synchronization!("secondary_core_entry()");
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
