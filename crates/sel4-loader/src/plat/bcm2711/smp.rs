use crate::drivers::spin_table;

const SPIN_TABLE: &[usize] = &[0xd8, 0xe0, 0xe8, 0xf0];

pub(crate) fn start_secondary_core(core_id: usize, sp: usize) {
    spin_table::start_secondary_core(SPIN_TABLE, core_id, sp)
}
