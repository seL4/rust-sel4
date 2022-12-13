use std::array;
use std::borrow::Borrow;
use std::ops::Range;

pub const NUM_ENTRIES: usize = 512;

const PAGE_BITS: u64 = 12;
const LEVEL_BITS: u64 = 9;
const NUM_LEVELS: u64 = 4;

pub const PHYS_BOUNDS: Range<u64> = 0..(1 << (LEVEL_BITS * NUM_LEVELS + PAGE_BITS));

pub struct Table {
    pub entries: [Entry; NUM_ENTRIES],
}

impl Table {
    pub fn construct(regions: impl Iterator<Item = impl Borrow<Region>>) -> Self {
        Construction::new(regions).construct()
    }
}

pub struct Entry {
    pub value: u64,
    pub child: Option<Box<Table>>,
}

impl Entry {
    fn leaf(value: u64) -> Self {
        Self::new(value, None)
    }

    fn branch(child: Box<Table>) -> Self {
        Self::new(0b11, Some(child))
    }

    fn new(value: u64, child: Option<Box<Table>>) -> Self {
        if child.is_some() {
            assert_eq!(value & (PHYS_BOUNDS.end - 1) & !(PAGE_BITS - 1), 0);
        }
        Self { value, child }
    }
}

pub struct Region {
    pub phys_range: Range<u64>,
    pub entries_valid_at_level_0: bool,
    pub mk_entry: Box<dyn Fn(u64) -> u64>,
}

impl Region {
    pub fn new(
        phys_range: Range<u64>,
        entries_valid_at_level_0: bool,
        mk_entry: impl 'static + Fn(u64) -> u64,
    ) -> Self {
        Self {
            phys_range,
            entries_valid_at_level_0,
            mk_entry: Box::new(mk_entry),
        }
    }

    pub fn valid(phys_range: Range<u64>, mk_entry: impl 'static + Fn(u64) -> u64) -> Self {
        Self::new(phys_range, false, mk_entry)
    }

    pub fn invalid(phys_range: Range<u64>) -> Self {
        Self::new(phys_range, true, |_| 0)
    }
}

struct Construction<T, U> {
    current: T,
    rest: U,
}

impl<T, U> Construction<T, U>
where
    T: Borrow<Region>,
    U: Iterator<Item = T>,
{
    fn new(mut regions: U) -> Self {
        let region = regions.next().unwrap();
        assert_eq!(region.borrow().phys_range.start, 0);
        assert!(region.borrow().phys_range.end <= PHYS_BOUNDS.end);
        Self {
            current: region,
            rest: regions,
        }
    }

    fn current_end(&self) -> u64 {
        self.current.borrow().phys_range.end
    }

    fn current_mk_entry(&self, vaddr: u64) -> u64 {
        (self.current.borrow().mk_entry)(vaddr)
    }

    fn current_entries_valid_at_level_0(&self) -> bool {
        self.current.borrow().entries_valid_at_level_0
    }

    fn advance(&mut self) -> bool {
        match self.rest.next() {
            None => {
                assert_eq!(self.current_end(), PHYS_BOUNDS.end);
                false
            }
            Some(region) => {
                assert_eq!(region.borrow().phys_range.start, self.current_end());
                assert!(region.borrow().phys_range.end <= PHYS_BOUNDS.end);
                self.current = region;
                true
            }
        }
    }

    fn construct(mut self) -> Table {
        let table = self.construct_inner(0, 0);
        assert!(!self.advance());
        table
    }

    fn construct_inner(&mut self, level: u64, table_vaddr: u64) -> Table {
        let step_bits = (NUM_LEVELS - level - 1) * LEVEL_BITS + PAGE_BITS;
        let step = 1 << step_bits;

        Table {
            entries: array::from_fn(|i| {
                let entry_vaddr = table_vaddr + u64::try_from(i).unwrap() * step;
                // println!("+ {} {:x} {:x}", level, entry_vaddr, self.current_end());
                while self.current_end() == entry_vaddr {
                    assert!(self.advance())
                }
                assert!(self.current_end() > entry_vaddr);
                if (level == 0 && !self.current_entries_valid_at_level_0())
                    || self.current_end() < entry_vaddr + step
                {
                    assert!(level < (NUM_LEVELS - 1));
                    Entry::branch(Box::new(self.construct_inner(level + 1, entry_vaddr)))
                } else {
                    Entry::leaf(self.current_mk_entry(entry_vaddr))
                }
            }),
        }
    }
}
