use super::*;
use proptest::prelude::*;

#[test]
fn rejects_empty_reordered_duplicate_and_out_of_range_maps() {
    assert_eq!(SlideMap::identity(0), Err(CoreError::EmptyDocument));
    for groups in [
        vec![vec![]],
        vec![vec![PhysicalPage(2)]],
        vec![vec![PhysicalPage(1), PhysicalPage(0)]],
        vec![vec![PhysicalPage(0)], vec![PhysicalPage(0)]],
    ] {
        assert_eq!(SlideMap::new(2, groups), Err(CoreError::InvalidSlideMap));
    }
    assert_eq!(SlideMap::new(2, vec![]), Err(CoreError::EmptyContent));
}
#[test]
fn boundaries_and_physical_clamp_skip_notes() {
    let map = SlideMap::new(
        8,
        vec![
            vec![PhysicalPage(1), PhysicalPage(3)],
            vec![PhysicalPage(6)],
        ],
    )
    .unwrap();
    assert_eq!(map.previous_step(PhysicalPage(1)), Some(PhysicalPage(1)));
    assert_eq!(map.next_step(PhysicalPage(6)), Some(PhysicalPage(6)));
    assert_eq!(map.next_step(PhysicalPage(2)), None);
    assert_eq!(
        map.closest_content_at_or_before(PhysicalPage(0)),
        PhysicalPage(1)
    );
    assert_eq!(
        map.closest_content_at_or_before(PhysicalPage(5)),
        PhysicalPage(3)
    );
    assert_eq!(
        map.closest_content_at_or_before(PhysicalPage(99)),
        PhysicalPage(6)
    );
    assert_eq!(map.next_logical(PhysicalPage(1)), Some(PhysicalPage(6)));
    assert_eq!(map.previous_logical(PhysicalPage(6)), Some(PhysicalPage(1)));
}
proptest! {
    #[test]
    fn neighbor_inverses_preserve_content_and_logical_groups(lengths in prop::collection::vec(1usize..8,1..30)) {
        let mut page=0;
        let groups=lengths.iter().map(|len| (0..*len).map(|_| {page+=2;PhysicalPage(page)}).collect()).collect();
        let map=SlideMap::new(page+2,groups).unwrap();
        for pair in map.steps().windows(2) {
            prop_assert_eq!(map.previous_step(map.next_step(pair[0]).unwrap()),Some(pair[0]));
            prop_assert_eq!(map.next_step(map.previous_step(pair[1]).unwrap()),Some(pair[1]));
        }
        for (i,g) in map.groups().iter().enumerate() {
            for &p in g {
                prop_assert!(p.0<map.page_count());
                if i+1<lengths.len() {prop_assert_eq!(map.logical_for(map.previous_logical(map.next_logical(p).unwrap()).unwrap()),Some(LogicalSlide(i)));}
                if i>0 {prop_assert_eq!(map.logical_for(map.next_logical(map.previous_logical(p).unwrap()).unwrap()),Some(LogicalSlide(i)));}
            }
        }
    }
}
