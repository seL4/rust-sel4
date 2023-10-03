use std::borrow::Borrow;
use std::sync::Arc;

use crate::regions::{AbstractRegion, AbstractRegions};
use crate::scheme::{Scheme, SchemeHelpers};

#[derive(Debug)]
pub struct Table<T: Scheme> {
    pub(crate) entries: Vec<AbstractEntry<T>>,
}

#[derive(Debug)]
pub(crate) enum AbstractEntry<T: Scheme> {
    Empty,
    Leaf(T::LeafDescriptor),
    Branch(Box<Table<T>>),
}

pub trait MkLeafFn<T: Scheme>: Fn(LeafLocation) -> T::LeafDescriptor {}

impl<T: Scheme, F: Fn(LeafLocation) -> T::LeafDescriptor> MkLeafFn<T> for F {}

pub struct LeafLocation {
    level: usize,
    vaddr: u64,
}

impl LeafLocation {
    pub fn level(&self) -> usize {
        self.level
    }

    pub fn vaddr(&self) -> u64 {
        self.vaddr
    }
}

pub struct RegionContent<T: Scheme> {
    mk_leaf: Box<dyn MkLeafFn<T>>,
}

impl<T: Scheme> RegionContent<T> {
    pub(crate) fn new(mk_leaf: impl MkLeafFn<T> + 'static) -> Self {
        Self {
            mk_leaf: Box::new(mk_leaf),
        }
    }

    fn mk_leaf(&self, level: usize, vaddr: u64) -> T::LeafDescriptor {
        (self.mk_leaf)(LeafLocation { level, vaddr })
    }
}

impl<T: Scheme> Table<T> {
    pub fn construct(regions: &AbstractRegions<Option<RegionContent<T>>>) -> Self {
        assert_eq!(regions.bounds(), SchemeHelpers::<T>::virt_bounds());
        Construction::new(regions.as_slice().iter()).construct()
    }
}

struct Construction<T, U, V> {
    marker: std::marker::PhantomData<T>,
    current: U,
    rest: V,
}

impl<T: Scheme, U, V> Construction<T, U, V>
where
    U: Borrow<AbstractRegion<Arc<Option<RegionContent<T>>>>>,
    V: Iterator<Item = U>,
{
    fn new(mut regions: V) -> Self {
        let region = regions.next().unwrap();
        Self {
            marker: std::marker::PhantomData,
            current: region,
            rest: regions,
        }
    }

    fn construct(mut self) -> Table<T> {
        let table = self.construct_inner(0, 0);
        assert!(!self.advance());
        table
    }

    fn construct_inner(&mut self, level: usize, table_vaddr: u64) -> Table<T> {
        assert!(level < T::NUM_LEVELS);
        let num_entries = SchemeHelpers::<T>::num_entries_in_table();
        let step_bits = (T::NUM_LEVELS - level - 1) * T::LEVEL_BITS + T::PAGE_BITS;
        let step = 1 << step_bits;
        Table {
            entries: (0..num_entries)
                .map(|i| {
                    let entry_vaddr = table_vaddr + u64::try_from(i).unwrap() * step;
                    while self.current_end() == entry_vaddr {
                        assert!(self.advance())
                    }
                    assert!(self.current_end() > entry_vaddr);
                    if self.current_end() < entry_vaddr + step
                        || (self.current_content().is_some() && level < T::MIN_LEVEL_FOR_LEAF)
                    {
                        AbstractEntry::Branch(Box::new(
                            self.construct_inner(level + 1, entry_vaddr),
                        ))
                    } else {
                        match self.current_content() {
                            None => AbstractEntry::Empty,
                            Some(region_content) => {
                                AbstractEntry::Leaf(region_content.mk_leaf(level, entry_vaddr))
                            }
                        }
                    }
                })
                .collect(),
        }
    }

    fn current(&self) -> &AbstractRegion<Arc<Option<RegionContent<T>>>> {
        self.current.borrow()
    }

    fn current_end(&self) -> u64 {
        self.current().range.end
    }

    fn current_content(&self) -> Option<&RegionContent<T>> {
        (*self.current().content).as_ref()
    }

    fn advance(&mut self) -> bool {
        self.rest
            .next()
            .map(|region| self.current = region)
            .is_some()
    }
}
