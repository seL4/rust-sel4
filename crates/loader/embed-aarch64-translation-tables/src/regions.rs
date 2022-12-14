use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractRegion<T> {
    pub range: Range<u64>,
    pub content: T,
}

impl<T> AbstractRegion<T> {
    pub fn new(range: Range<u64>, content: T) -> Self {
        Self { range, content }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractRegions<T> {
    regions: Vec<AbstractRegion<T>>,
}

impl<T> AbstractRegions<T> {
    pub fn new_with(background: AbstractRegion<T>) -> Self {
        Self {
            regions: vec![background],
        }
    }

    pub fn as_slice(&self) -> &[AbstractRegion<T>] {
        &self.regions
    }
}

impl<T: Clone> AbstractRegions<T> {
    pub fn insert(self, region: AbstractRegion<T>) -> Self {
        let regions = self.regions;
        let i_left = regions
            .partition_point(|existing_region| existing_region.range.end <= region.range.start);
        let i_right = regions
            .partition_point(|existing_region| existing_region.range.start < region.range.end)
            - 1;
        let mut left = regions[i_left].clone();
        let mut right = regions[i_right].clone();
        left.range.end = region.range.start;
        right.range.start = region.range.end;
        let new_regions = regions[..i_left]
            .iter()
            .chain(
                [left, region, right]
                    .iter()
                    .filter(|r| r.range.start < r.range.end),
            )
            .chain(regions[i_right + 1..].iter())
            .cloned()
            .collect::<Vec<AbstractRegion<T>>>();
        Self {
            regions: new_regions,
        }
    }

    #[allow(dead_code)]
    fn check(&self) {
        assert!(self.regions.len() > 0);
        for region in self.regions.iter() {
            assert!(region.range.start < region.range.end);
        }
        for window in self.regions.windows(2) {
            assert_eq!(window[0].range.end, window[1].range.start);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        #[allow(unused_variables)]
        let show = |regions: &AbstractRegions<usize>| {
            for region in regions.as_slice() {
                eprint!("{:?} ({:?}), ", region.range, region.content);
            }
            eprintln!("");
        };
        let regions = AbstractRegions::<usize>::new_with(AbstractRegion::new(0..10, 0));
        regions.check();
        let regions = regions.insert(AbstractRegion::new(0..1, 1));
        regions.check();
        let regions = regions.insert(AbstractRegion::new(0..2, 2));
        regions.check();
        let regions = regions.insert(AbstractRegion::new(2..5, 3));
        regions.check();
        let regions = regions.insert(AbstractRegion::new(1..4, 4));
        regions.check();
        let regions = regions.insert(AbstractRegion::new(7..9, 5));
        regions.check();
    }
}
