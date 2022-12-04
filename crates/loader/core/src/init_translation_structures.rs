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

        boot_lvl0_lower.entries[0] = boot_lvl1_lower_addr | bit(1) | bit(0);

        for i in 0..512 {
            boot_lvl1_lower.entries[i] =
                ((i as u64) << ARM_1GB_BLOCK_BITS) | bit(10) | (0u64 << 2) | bit(0);
        }

        boot_lvl0_lower.entries[get_pgd_index(kernel_virt_start as u64) as usize] =
            boot_lvl1_upper_addr | bit(1) | bit(0);
        boot_lvl1_upper.entries[get_pud_index(kernel_virt_start as u64) as usize] =
            boot_lvl2_upper_addr | bit(1) | bit(0);

        let pmd_index = get_pmd_index(kernel_virt_start as u64);
        for i in (pmd_index as usize)..512 {
            boot_lvl2_upper.entries[i] = ((((i as u64) - pmd_index) << ARM_2MB_BLOCK_BITS) + (kernel_phys_start as u64))
                    | bit(10) /* access flag */

                    | (4 << 2) /* MT_NORMAL memory */
                    | bit(0); /* 2M block */
            sel4_config::sel4_cfg_if! {
                if #[cfg(not(MAX_NUM_NODES = "1"))] {
                    boot_lvl2_upper.entries[i] |= 3 << 8;
                }
            }
        }
    }
}

//

#[repr(C, align(4096))]
struct TranslationStructure {
    entries: [u64; 512],
}

fn bit(i: u64) -> u64 {
    1 << i
}

const ARM_1GB_BLOCK_BITS: u64 = 30;
const ARM_2MB_BLOCK_BITS: u64 = 21;

fn get_pgd_index(addr: u64) -> u64 {
    (addr >> (12 + 9 + 9 + 9)) & ((1 << 9) - 1)
}

fn get_pud_index(addr: u64) -> u64 {
    (addr >> (12 + 9 + 9)) & ((1 << 9) - 1)
}

fn get_pmd_index(addr: u64) -> u64 {
    (addr >> (12 + 9)) & ((1 << 9) - 1)
}
