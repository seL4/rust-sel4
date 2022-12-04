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
    // HACK
    fn call_psci(function_id: u32, target_cpu: u64, entry_point: u64, context_id: u64) -> u64;
}
