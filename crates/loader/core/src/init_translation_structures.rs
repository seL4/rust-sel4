#![allow(dead_code)]
#![allow(unused_variables)]

const PAGE_BITS: u64 = 12;
const LEVEL_BITS: u64 = 9;

const ARM_1GB_BLOCK_BITS: u64 = 30;
const ARM_2MB_BLOCK_BITS: u64 = 21;
const ARM_4KB_BLOCK_BITS: u64 = 12;

const fn bit(i: u64) -> u64 {
    1 << i
}

const PGD_LEVEL_FROM_LSB: u64 = 3;
const PUD_LEVEL_FROM_LSB: u64 = 2;
const PD_LEVEL_FROM_LSB: u64 = 1;
const PT_LEVEL_FROM_LSB: u64 = 0;

const fn get_level_index(level_from_lsb: u64, addr: u64) -> u64 {
    (addr >> (LEVEL_BITS * level_from_lsb + PAGE_BITS)) & ((1 << LEVEL_BITS) -1)
}

const fn get_pgd_index(addr: u64) -> u64 {
    get_level_index(PGD_LEVEL_FROM_LSB, addr)
}

const fn get_pud_index(addr: u64) -> u64 {
    get_level_index(PUD_LEVEL_FROM_LSB, addr)
}

const fn get_pd_index(addr: u64) -> u64 {
    get_level_index(PD_LEVEL_FROM_LSB, addr)
}

//

#[repr(C, align(4096))]
struct TranslationStructure {
    entries: [u64; 512],
}

#[used]
#[no_mangle]
static mut boot_lvl0_upper: TranslationStructure = TranslationStructure { entries: [0; 512] };

#[used]
#[no_mangle]
static mut boot_lvl1_upper: TranslationStructure = TranslationStructure { entries: [0; 512] };

#[used]
#[no_mangle]
static mut boot_lvl2_upper: TranslationStructure = TranslationStructure { entries: [0; 512] };

#[used]
#[no_mangle]
static mut boot_lvl0_lower: TranslationStructure = TranslationStructure { entries: [0; 512] };

#[used]
#[no_mangle]
static mut boot_lvl1_lower: TranslationStructure = TranslationStructure { entries: [0; 512] };

pub fn init_translation_structures(kernel_phys_start: usize, kernel_virt_start: usize) {
    unsafe {
        let boot_lvl1_lower_addr = &boot_lvl1_lower as *const TranslationStructure as u64;
        let boot_lvl1_upper_addr = &boot_lvl1_upper as *const TranslationStructure as u64;
        let boot_lvl2_upper_addr = &boot_lvl2_upper as *const TranslationStructure as u64;

        boot_lvl0_lower.entries[0] = 0
            | boot_lvl1_lower_addr
            | bit(1) | bit(0) // valid
        ;

        for i in 0..512 {
            boot_lvl1_lower.entries[i] = 0
                | ((i as u64) << ARM_1GB_BLOCK_BITS)
                | bit(10) // access flag
                | (0 << 2) // select MT_DEVICE_nGnRnE
                | bit(0) // mark as valid
            ;
        }

        boot_lvl0_lower.entries[get_pgd_index(kernel_virt_start as u64) as usize] = 0
            | boot_lvl1_upper_addr
            | bit(1) | bit(0) // mark as valid
        ;

        boot_lvl1_upper.entries[get_pud_index(kernel_virt_start as u64) as usize] = 0
            | boot_lvl2_upper_addr
            | bit(1) | bit(0) // mark as valid
        ;

        let pd_index = get_pd_index(kernel_virt_start as u64) as usize;
        for i in pd_index..512 {
            let mut entry = 0
                | ((((i - pd_index) as u64) << ARM_2MB_BLOCK_BITS) + (kernel_phys_start as u64))
                | bit(10) // access flag
                | (4 << 2) // select MT_NORMAL
                | bit(0) // mark as valid
            ;
            if sel4_config::sel4_cfg_usize!(MAX_NUM_NODES) > 1 {
                entry |= 3 << 8;
            }
            boot_lvl2_upper.entries[i] = entry;
        }
    }
}
