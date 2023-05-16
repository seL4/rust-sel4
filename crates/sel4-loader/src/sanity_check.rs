use core::borrow::Borrow;
use core::ops::Range;

use sel4_loader_payload_types::Region;
use sel4_platform_info::PLATFORM_INFO;

pub(crate) fn sanity_check<T>(own_footprint: &Range<u64>, regions: &[Region<T>]) {
    let memory = &PLATFORM_INFO.memory;
    assert!(any_range_contains(memory.iter(), own_footprint));
    for region in regions.iter() {
        let region = region.borrow();
        assert!(any_range_contains(memory.iter(), &region.phys_addr_range));
        assert!(ranges_are_disjoint(own_footprint, &region.phys_addr_range));
    }
}

fn range_contains(this: &Range<u64>, that: &Range<u64>) -> bool {
    this.start <= that.start && that.end <= this.end
}

fn ranges_are_disjoint(this: &Range<u64>, that: &Range<u64>) -> bool {
    this.end.min(that.end) <= this.start.max(that.start)
}

fn any_range_contains<'a>(
    mut these: impl Iterator<Item = &'a Range<u64>>,
    that: &Range<u64>,
) -> bool {
    these.any(|this| range_contains(this, that))
}
