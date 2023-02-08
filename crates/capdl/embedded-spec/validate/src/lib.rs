#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::fmt;

use capdl_types::*;

type SpecCommon<'a, N> = Spec<'a, N, Vec<u8>>;

pub fn run() {
    compare(
        &capdl_embedded_spec_serialized::get(),
        &capdl_embedded_spec::SPEC,
    )
}

fn compare<'a, N: ObjectNameForComparison, F: SelfContainedContent>(
    serialized: &Spec<'a, String, (FileContent, BytesContent<'static>)>,
    embedded: &Spec<'a, SelfContained<N>, SelfContained<F>>,
) where
    N::ComparisonPoint: fmt::Debug,
{
    let serialized = adapt_serialized::<N>(serialized);
    let embedded = adapt_embedded(embedded);
    if serialized != embedded {
        // NOTE for debugging:
        // std::fs::write("embedded.txt", format!("{:#?}", &embedded)).unwrap();
        // std::fs::write("serialized.txt", format!("{:#?}", &serialized)).unwrap();
        panic!("not equal");
    }
}

fn adapt_embedded<'a, N: ObjectNameForComparison, F: SelfContainedContent>(
    spec: &Spec<'a, SelfContained<N>, SelfContained<F>>,
) -> SpecCommon<'a, N::ComparisonPoint> {
    spec.traverse(
        |_object, name| Ok(name.inner().us()),
        |length, content| {
            let mut v = vec![0; length];
            content.inner().self_contained_copy_out(&mut v);
            Ok::<_, !>(v)
        },
    )
    .into_ok()
}

fn adapt_serialized<'a, N: ObjectNameForComparison>(
    spec: &Spec<'a, String, (FileContent, BytesContent<'static>)>,
) -> SpecCommon<'a, N::ComparisonPoint> {
    spec.traverse(
        |_object, name| Ok(N::them(name)),
        |_length, content| Ok::<_, !>(content.1.bytes.to_vec()),
    )
    .into_ok()
}

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

// TODO
impl ObjectNameForComparison for Option<&str> {
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
