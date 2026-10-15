/// Monotonic id of a loaded document generation.
///
/// A hot reload publishes a strictly greater generation. In-flight renders that
/// still carry the old id are dropped instead of being drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentGeneration(pub u64);

/// Zero-based index of a physical PDF page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhysicalPage(pub usize);

#[cfg(test)]
mod order_tests {
    use super::DocumentGeneration;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn generation_order_matches_the_inner_integer(a in 0u64..1_000, b in 0u64..1_000) {
            prop_assert_eq!(
                DocumentGeneration(a).cmp(&DocumentGeneration(b)),
                a.cmp(&b)
            );
        }
    }
}

/// Zero-based index of a logical Beamer slide.
///
/// A logical group contains one or more navigable content steps in SlideMap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LogicalSlide(pub usize);
