#![no_std]
#![feature(const_pointer_byte_offsets)]
#![feature(pointer_byte_offsets)]

use core::marker::PhantomData;

pub trait Scheme<const NUM_ENTRIES: usize> {}

pub enum AArch64 {}

impl Scheme<512> for AArch64 {}

pub enum RiscV64 {}

impl Scheme<512> for RiscV64 {}

pub trait RiscVScheme {}

impl RiscVScheme for RiscV64 {}

const RISCV_SCHEME_ROTATE_RIGHT_FOR_FINISH: u32 = 2;

#[repr(C)]
pub struct Tables<T: Scheme<NUM_ENTRIES>, const NUM_ENTRIES: usize, const NUM_TABLES: usize> {
    tables: [Table<T, NUM_ENTRIES>; NUM_TABLES],
}

impl<T: Scheme<NUM_ENTRIES>, const NUM_ENTRIES: usize, const NUM_TABLES: usize>
    Tables<T, NUM_ENTRIES, NUM_TABLES>
{
    pub const fn new(tables: [Table<T, NUM_ENTRIES>; NUM_TABLES]) -> Self {
        Self { tables }
    }

    pub const fn table(&self, index: usize) -> *const () {
        &self.tables[index] as *const _ as *const ()
    }

    pub const fn root(&self) -> *const () {
        self.table(0)
    }
}

impl<T: Scheme<NUM_ENTRIES> + RiscVScheme, const NUM_ENTRIES: usize, const NUM_TABLES: usize>
    Tables<T, NUM_ENTRIES, NUM_TABLES>
{
    pub fn finish(&mut self) {
        for table in self.tables.iter_mut() {
            for entry in table.entries.iter_mut() {
                *entry = entry.rotate_right(RISCV_SCHEME_ROTATE_RIGHT_FOR_FINISH);
            }
        }
    }
}

#[repr(C, align(4096))]
pub struct Table<T: Scheme<NUM_ENTRIES>, const NUM_ENTRIES: usize> {
    _phantom: PhantomData<T>,
    entries: [Entry; NUM_ENTRIES],
}

impl<T: Scheme<NUM_ENTRIES>, const NUM_ENTRIES: usize> Table<T, NUM_ENTRIES> {
    pub const fn new(entries: [Entry; NUM_ENTRIES]) -> Self {
        Self {
            _phantom: PhantomData,
            entries,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Entry(*const ());

impl Entry {
    pub const fn new(ptr: Option<*const ()>, offset: usize) -> Self {
        Self(match ptr {
            Some(ptr) => unsafe { ptr.byte_add(offset) },
            None => offset as *const (),
        })
    }

    fn rotate_right(self, n: u32) -> Self {
        Self((self.0 as usize).rotate_right(n) as *const ())
    }
}

pub enum Test {}

impl Scheme<1> for Test {}

impl RiscVScheme for Test {}

#[no_mangle]
pub static mut these_tables: Tables<Test, 1, 2> = Tables::new(unsafe {
    [
        // Table::new([&these_tables.tables[1] as *const Table<A4096, 1> as *const ()]),
        Table::new([Entry::new(Some(these_tables.table(1)), 1)]),
        Table::new([Entry::new(None, 0)]),
    ]
});
