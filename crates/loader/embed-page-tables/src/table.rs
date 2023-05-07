use std::array;
use std::borrow::Borrow;
use std::ops::Range;
use std::sync::Arc;

use crate::regions::AbstractRegion;

pub const NUM_ENTRIES: usize = 512;

const PAGE_BITS: u64 = 12;
const LEVEL_BITS: u64 = 9;
const NUM_LEVELS: u64 = 4;

pub const PHYS_BOUNDS: Range<u64> = 0..(1 << (LEVEL_BITS * NUM_LEVELS + PAGE_BITS));

pub struct Table<T> {
    pub entries: [AbstractEntry<T>; NUM_ENTRIES],
}

impl<T> Table<T> {
    pub fn construct(
        regions: impl Iterator<Item = impl Borrow<AbstractRegion<RegionContent<T>>>>,
    ) -> Self {
        Construction::new(regions).construct()
    }
}

pub enum AbstractEntry<T> {
    Leaf(T),
    Branch(Box<Table<T>>),
}

#[derive(Clone)]
pub struct RegionContent<T> {
    pub min_level_for_leaf: u64,
    pub mk_leaf: Arc<Box<dyn Fn(MkLeafFnParams) -> T>>,
}

pub struct MkLeafFnParams {
    pub level: u64,
    pub vaddr: u64,
}

struct Construction<T, U, V> {
    marker: std::marker::PhantomData<V>,
    current: T,
    rest: U,
}

impl<T, U, V> Construction<T, U, V>
where
    T: Borrow<AbstractRegion<RegionContent<V>>>,
    U: Iterator<Item = T>,
{
    fn new(mut regions: U) -> Self {
        let region = regions.next().unwrap();
        assert_eq!(region.borrow().range.start, 0);
        assert!(region.borrow().range.end <= PHYS_BOUNDS.end);
        Self {
            marker: std::marker::PhantomData,
            current: region,
            rest: regions,
        }
    }

    fn construct(mut self) -> Table<V> {
        let table = self.construct_inner(0, 0);
        assert!(!self.advance());
        table
    }

    fn construct_inner(&mut self, level: u64, table_vaddr: u64) -> Table<V> {
        let step_bits = (NUM_LEVELS - level - 1) * LEVEL_BITS + PAGE_BITS;
        let step = 1 << step_bits;

        Table {
            entries: array::from_fn(|i| {
                let entry_vaddr = table_vaddr + u64::try_from(i).unwrap() * step;
                while self.current_end() == entry_vaddr {
                    assert!(self.advance())
                }
                assert!(self.current_end() > entry_vaddr);
                if level < self.current_min_level_for_leaf()
                    || self.current_end() < entry_vaddr + step
                {
                    assert!(level < (NUM_LEVELS - 1));
                    AbstractEntry::Branch(Box::new(self.construct_inner(level + 1, entry_vaddr)))
                } else {
                    AbstractEntry::Leaf(self.current_mk_leaf(level, entry_vaddr))
                }
            }),
        }
    }

    fn current_end(&self) -> u64 {
        self.current.borrow().range.end
    }

    fn current_mk_leaf(&self, level: u64, vaddr: u64) -> V {
        (self.current.borrow().content.mk_leaf)(MkLeafFnParams { level, vaddr })
    }

    fn current_min_level_for_leaf(&self) -> u64 {
        self.current.borrow().content.min_level_for_leaf
    }

    fn advance(&mut self) -> bool {
        match self.rest.next() {
            None => {
                assert_eq!(self.current_end(), PHYS_BOUNDS.end);
                false
            }
            Some(region) => {
                assert_eq!(region.borrow().range.start, self.current_end());
                assert!(region.borrow().range.end <= PHYS_BOUNDS.end);
                self.current = region;
                true
            }
        }
    }
}
