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

type SpecCommon<'a, N> = Spec<'a, N, Vec<u8>>;

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

fn compare<'a, N: ObjectNameForComparison, F: AvailableFillEntryContent>(
    serialized: &Spec<'a, String, (FillEntryContentFile, FillEntryContentBytes<'static>)>,
    embedded: &Spec<'a, N, F>,
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

fn translate_embedded<'a, N: ObjectNameForComparison, F: AvailableFillEntryContent>(
    spec: &Spec<'a, N, F>,
) -> SpecCommon<'a, N::ComparisonPoint> {
    spec.traverse(
        |name| Ok(name.us()),
        |length, content| {
            let mut v = vec![0; length];
            content.copy_out(&mut v);
            Ok::<_, !>(v)
        },
    )
    .into_ok()
}

fn translate_serialized<'a, N: ObjectNameForComparison>(
    spec: &Spec<'a, String, (FillEntryContentFile, FillEntryContentBytes<'static>)>,
) -> SpecCommon<'a, N::ComparisonPoint> {
    spec.traverse(
        |name| Ok(N::them(name)),
        |_length, content| Ok::<_, !>(content.1.bytes.to_vec()),
    )
    .into_ok()
}
