#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fmt;

use capdl_types::*;

pub fn run() {
    compare(
        &capdl_embedded_spec_serialized::get(),
        &capdl_embedded_spec::SPEC,
    )
}

type SpecCommon<'a, S> = ConcreteSpec<'a, VecContainer, Fill, S>;
type Fill = Vec<u8>;

trait ObjectNameForComparison {
    type ComparisonPoint: Clone + Eq + PartialEq;

    fn us(&self) -> Self::ComparisonPoint;
    fn them(them: &str) -> Self::ComparisonPoint;
}

impl ObjectNameForComparison for Unnamed {
    type ComparisonPoint = ();

    fn us(&self) -> Self::ComparisonPoint {
        ()
    }

    fn them(_them: &str) -> Self::ComparisonPoint {
        ()
    }
}

impl ObjectNameForComparison for &str {
    type ComparisonPoint = String;

    fn us(&self) -> Self::ComparisonPoint {
        (*self).to_owned()
    }

    fn them(them: &str) -> Self::ComparisonPoint {
        them.to_owned()
    }
}

fn compare<'a, F: AvailableFillEntryContent, N: ObjectNameForComparison>(
    serialized: &SpecForBuildSystem<'a, (FillEntryContentFile, FillEntryContentBytes<'static>)>,
    embedded: &'static SpecForLoader<'a, F, N>,
) where
    N::ComparisonPoint: fmt::Debug,
{
    let serialized = translate_serialized::<N>(serialized);
    let embedded = translate_embedded(embedded);
    if serialized != embedded {
        // HACK
        // std::fs::write("embedded.txt", format!("{:#?}", &embedded)).unwrap();
        // std::fs::write("serialized.txt", format!("{:#?}", &serialized)).unwrap();
        panic!("not equal");
    }
}

fn translate_embedded<'a, F: AvailableFillEntryContent, N: ObjectNameForComparison>(
    spec: &'static SpecForLoader<'a, F, N>,
) -> SpecCommon<'a, N::ComparisonPoint> {
    spec.traverse(
        |length, content| {
            let mut v = vec![0; length];
            content.copy_out(&mut v);
            Ok::<_, !>(v)
        },
        |name| Ok(name.us()),
    )
    .into_ok()
}

fn translate_serialized<'a, N: ObjectNameForComparison>(
    spec: &SpecForBuildSystem<'a, (FillEntryContentFile, FillEntryContentBytes<'static>)>,
) -> SpecCommon<'a, N::ComparisonPoint> {
    spec.traverse(
        |_length, content| Ok::<_, !>(content.1.bytes.to_vec()),
        |name| Ok(N::them(name)),
    )
    .into_ok()
}
