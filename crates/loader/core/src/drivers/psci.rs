use core::arch::global_asm;

pub fn cpu_on(
    target_cpu: u64,
    entry_point: u64,
    context_id: u64,
) -> Result<(), psci::error::Error> {
    success_or_error_64(unsafe {
        call_psci(psci::PSCI_CPU_ON_64, target_cpu, entry_point, context_id)
    })
}

fn success_or_error_64(value: u64) -> Result<(), psci::error::Error> {
    success_or_error(value as i32)
}

fn success_or_error(value: i32) -> Result<(), psci::error::Error> {
    if value == psci::error::SUCCESS {
        Ok(())
    } else {
        Err(value.into())
    }
}

extern "C" {
    fn call_psci(function_id: u32, target_cpu: u64, entry_point: u64, context_id: u64) -> u64;
}

//

pub(crate) fn start_secondary_core(core_id: usize, sp: usize) {
    let start = (psci_secondary_core_entry_without_sp as *const SecondaryCoreStartFn).to_bits();
    cpu_on(
        core_id.try_into().unwrap(),
        start.try_into().unwrap(),
        sp.try_into().unwrap(),
    )
    .unwrap();
}

type SecondaryCoreStartFn = extern "C" fn() -> !;

extern "C" {
    fn psci_secondary_core_entry_without_sp() -> !;
}

global_asm! {
    r#"
        .global psci_secondary_core_entry_without_sp
        .extern secondary_entry

        .section .text

        psci_secondary_core_entry_without_sp:
            mov sp, x0
            b secondary_entry
    "#
}
