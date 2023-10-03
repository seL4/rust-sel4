use std::ops::Range;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractRegion<T> {
    pub(crate) range: Range<u64>,
    pub(crate) content: T,
}

impl<T> AbstractRegion<T> {
    pub fn new(range: Range<u64>, content: T) -> Self {
        Self { range, content }
    }

    fn into_arc(self) -> AbstractRegion<Arc<T>> {
        AbstractRegion {
            range: self.range,
            content: Arc::new(self.content),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractRegionsBuilder<T> {
    regions: Vec<AbstractRegion<Arc<T>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractRegions<T> {
    checked: AbstractRegionsBuilder<T>,
}

impl<T> AbstractRegionsBuilder<T> {
    pub fn new_with_background(background: AbstractRegion<T>) -> Self {
        Self {
            regions: vec![background.into_arc()],
        }
    }

    fn bounds(&self) -> Range<u64> {
        let start = self.regions.first().unwrap().range.start;
        let end = self.regions.last().unwrap().range.end;
        start..end
    }
}

impl<T> AbstractRegionsBuilder<T> {
    pub fn insert(self, region: AbstractRegion<T>) -> Self {
        {
            let bounds = self.bounds();
            assert!(bounds.start <= region.range.start);
            assert!(bounds.end >= region.range.end);
        }

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
                [left, region.into_arc(), right]
                    .iter()
                    .filter(|r| r.range.start < r.range.end),
            )
            .chain(regions[i_right + 1..].iter())
            .cloned()
            .collect::<Vec<AbstractRegion<Arc<T>>>>();

        Self {
            regions: new_regions,
        }
    }
}

impl<T> AbstractRegionsBuilder<T> {
    fn check(&self) {
        assert!(!self.regions.is_empty());
        for region in self.regions.iter() {
            assert!(region.range.start < region.range.end);
        }
        for window in self.regions.windows(2) {
            assert_eq!(window[0].range.end, window[1].range.start);
        }
    }

    pub fn build(self) -> AbstractRegions<T> {
        self.check();
        AbstractRegions { checked: self }
    }
}

impl<T> AbstractRegions<T> {
    pub fn as_slice(&self) -> &[AbstractRegion<Arc<T>>] {
        &self.checked.regions
    }

    pub fn bounds(&self) -> Range<u64> {
        self.checked.bounds()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[allow(dead_code)]
    fn show(regions: &AbstractRegions<usize>) {
        for region in regions.as_slice() {
            eprint!("{:?} ({:?}), ", region.range, region.content);
        }
        eprintln!("");
    }

    #[test]
    fn test() {
        let build =
            AbstractRegionsBuilder::<usize>::new_with_background(AbstractRegion::new(0..10, 0));
        build.clone().build();
        let build = build.insert(AbstractRegion::new(0..1, 1));
        build.clone().build();
        let build = build.insert(AbstractRegion::new(0..2, 2));
        build.clone().build();
        let build = build.insert(AbstractRegion::new(2..5, 3));
        build.clone().build();
        let build = build.insert(AbstractRegion::new(1..4, 4));
        build.clone().build();
        let build = build.insert(AbstractRegion::new(7..9, 5));
        build.clone().build();
    }
}
