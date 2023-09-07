use core::ops::Range;

use sel4_kernel_loader_payload_types::Region;
use sel4_platform_info::PLATFORM_INFO;

#[cfg(target_pointer_width = "32")]
pub type Word = u32;

#[cfg(target_pointer_width = "64")]
pub type Word = u64;

pub(crate) fn sanity_check<T>(own_footprint: &Range<usize>, regions: &[Region<usize, T>]) {
    let memory = &PLATFORM_INFO.memory;
    assert!(any_range_contains(memory.iter(), own_footprint));
    for region in regions.iter() {
        assert!(any_range_contains(memory.iter(), &region.phys_addr_range));
        assert!(ranges_are_disjoint(own_footprint, &region.phys_addr_range));
    }
}

fn range_contains(this: &Range<Word>, that: &Range<usize>) -> bool {
    usize::try_from(this.start).unwrap() <= that.start
        && that.end <= usize::try_from(this.end).unwrap()
}

fn ranges_are_disjoint(this: &Range<usize>, that: &Range<usize>) -> bool {
    this.end.min(that.end) <= this.start.max(that.start)
}

fn any_range_contains<'a>(
    mut these: impl Iterator<Item = &'a Range<Word>>,
    that: &Range<usize>,
) -> bool {
    these.any(|this| range_contains(this, that))
}
