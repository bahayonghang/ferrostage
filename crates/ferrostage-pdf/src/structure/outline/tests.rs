use super::*;
use lopdf::{StringFormat, dictionary};

struct Fixture {
    document: Document,
    pages: Vec<ObjectId>,
    catalog: ObjectId,
    outlines: ObjectId,
}

impl Fixture {
    fn new() -> Self {
        let mut document = Document::with_version("1.7");
        let tree = document.new_object_id();
        let pages: Vec<_> = (0..6)
            .map(|_| {
                document.add_object(dictionary! {
                    "Type" => "Page", "Parent" => tree,
                    "MediaBox" => vec![0.into(), 0.into(), 320.into(), 180.into()],
                })
            })
            .collect();
        document.set_object(
            tree,
            dictionary! {
                "Type" => "Pages", "Count" => pages.len() as i64,
                "Kids" => pages.iter().copied().map(Object::Reference).collect::<Vec<_>>()
            },
        );
        let outlines = document.add_object(dictionary! { "Type" => "Outlines" });
        let catalog = document.add_object(dictionary! {
            "Type" => "Catalog", "Pages" => tree, "Outlines" => outlines
        });
        document.trailer.set("Root", catalog);
        Self {
            document,
            pages,
            catalog,
            outlines,
        }
    }

    fn destination(&self, page: usize) -> Object {
        Object::Array(vec![self.pages[page].into(), Object::Name(b"Fit".to_vec())])
    }

    fn entry(&mut self, title: &str, target: Option<Object>) -> ObjectId {
        let mut bytes = vec![0xfe, 0xff];
        bytes.extend(title.encode_utf16().flat_map(u16::to_be_bytes));
        let mut entry = dictionary! { "Title" => Object::String(bytes, StringFormat::Hexadecimal) };
        if let Some(target) = target {
            entry.set("Dest", target);
        }
        self.document.add_object(entry)
    }

    fn link(&mut self, from: ObjectId, key: &str, to: ObjectId) {
        self.document.get_dictionary_mut(from).unwrap().set(key, to);
    }

    fn read(&mut self) -> PdfOutline {
        let mut bytes = Vec::new();
        self.document.save_to(&mut bytes).unwrap();
        let structure = crate::extract_structure(&bytes).unwrap();
        assert_eq!(
            structure.pages.len(),
            6,
            "bad outlines cannot break page loading"
        );
        structure.outline
    }
}

#[test]
fn retains_source_order_unicode_unnumbered_duplicates_and_only_two_levels() {
    let mut f = Fixture::new();
    let cover = f.entry("封面", Some(f.destination(0)));
    let chapter = f.entry("第2章", Some(f.destination(1)));
    let section = f.entry("2.2 水泥烧成", Some(f.destination(2)));
    let deep = f.entry("2.2.1 hidden", Some(f.destination(3)));
    let conclusion = f.entry("本章小结", Some(f.destination(2)));
    let thanks = f.entry("致谢", Some(f.destination(5)));
    f.link(f.outlines, "First", cover);
    f.link(cover, "Next", chapter);
    f.link(chapter, "First", section);
    f.link(chapter, "Next", thanks);
    f.link(section, "First", deep);
    f.link(section, "Next", conclusion);
    // A malicious cycle below the chosen depth is never traversed.
    f.link(deep, "Next", deep);
    let outline = f.read();
    assert!(!outline.partial);
    assert_eq!(
        outline
            .entries
            .iter()
            .map(|e| e.title.as_deref().unwrap())
            .collect::<Vec<_>>(),
        ["封面", "第2章", "2.2 水泥烧成", "本章小结", "致谢"]
    );
    assert_eq!(
        outline
            .entries
            .iter()
            .map(|e| (e.id, e.parent, e.depth, e.page))
            .collect::<Vec<_>>(),
        [
            (0, None, 1, Some(PhysicalPage(0))),
            (1, None, 1, Some(PhysicalPage(1))),
            (2, Some(1), 2, Some(PhysicalPage(2))),
            (3, Some(1), 2, Some(PhysicalPage(2))),
            (4, None, 1, Some(PhysicalPage(5))),
        ]
    );
}

#[test]
fn local_goto_direct_indirect_catalog_and_name_tree_destinations() {
    let mut f = Fixture::new();
    let direct = f.document.add_object(f.destination(2));
    let wrapped = f.document.add_object(dictionary! { "D" => direct });
    let leaf = f.document.add_object(dictionary! {
        "Names" => vec![Object::string_literal("tree"), wrapped.into()]
    });
    let root = f
        .document
        .add_object(dictionary! { "Kids" => vec![Object::Reference(leaf)] });
    let names = f.document.add_object(dictionary! { "Dests" => root });
    let legacy = f
        .document
        .add_object(dictionary! { "legacy" => f.destination(4) });
    f.document
        .get_dictionary_mut(f.catalog)
        .unwrap()
        .set("Names", names);
    f.document
        .get_dictionary_mut(f.catalog)
        .unwrap()
        .set("Dests", legacy);
    let first = f.entry("indirect", Some(direct.into()));
    let tree = f.entry("tree", Some(Object::string_literal("tree")));
    let legacy = f.entry("legacy", Some(Object::Name(b"legacy".to_vec())));
    let action = f.entry("action", None);
    let action_dict = f
        .document
        .add_object(dictionary! { "S" => "GoTo", "D" => wrapped });
    f.document
        .get_dictionary_mut(action)
        .unwrap()
        .set("A", action_dict);
    f.link(f.outlines, "First", first);
    f.link(first, "Next", tree);
    f.link(tree, "Next", legacy);
    f.link(legacy, "Next", action);
    let outline = f.read();
    assert!(!outline.partial);
    assert_eq!(
        outline.entries.iter().map(|e| e.page).collect::<Vec<_>>(),
        [
            Some(PhysicalPage(2)),
            Some(PhysicalPage(2)),
            Some(PhysicalPage(4)),
            Some(PhysicalPage(2))
        ]
    );
}

#[test]
fn malformed_parent_title_and_target_keep_valid_children_and_siblings() {
    let mut f = Fixture::new();
    let parent = f.entry("bad", Some(Object::Array(vec![])));
    f.document
        .get_dictionary_mut(parent)
        .unwrap()
        .set("Title", 7);
    let child = f.entry("valid child", Some(f.destination(3)));
    let sibling = f.entry("valid sibling", Some(f.destination(5)));
    f.link(f.outlines, "First", parent);
    f.link(parent, "First", child);
    f.link(parent, "Next", sibling);
    let outline = f.read();
    assert!(outline.partial);
    assert_eq!(outline.entries.len(), 3);
    assert_eq!(outline.entries[0].title, None);
    assert_eq!(outline.entries[0].issue, Some(PdfOutlineIssue::Unavailable));
    assert_eq!(outline.entries[1].parent, Some(0));
    assert_eq!(outline.entries[1].page, Some(PhysicalPage(3)));
    assert_eq!(outline.entries[2].page, Some(PhysicalPage(5)));
}

#[test]
fn missing_short_unknown_and_nonpage_destinations_are_disabled() {
    let mut f = Fixture::new();
    let nonpage = f
        .document
        .add_object(dictionary! { "Type" => "SomethingElse" });
    let targets = [
        None,
        Some(Object::Array(vec![])),
        Some(Object::Array(vec![f.pages[0].into()])),
        Some(Object::Array(vec![
            f.pages[0].into(),
            Object::Name(b"XYZ".to_vec()),
        ])),
        Some(Object::Array(vec![
            999.into(),
            Object::Name(b"Fit".to_vec()),
        ])),
        Some(Object::Array(vec![
            nonpage.into(),
            Object::Name(b"Fit".to_vec()),
        ])),
        Some(Object::Array(vec![
            Object::Reference((99999, 0)),
            Object::Name(b"Fit".to_vec()),
        ])),
        Some(Object::Name(b"missing".to_vec())),
    ];
    let mut previous = f.outlines;
    for (index, target) in targets.into_iter().enumerate() {
        let entry = f.entry("invalid", target);
        f.link(previous, if index == 0 { "First" } else { "Next" }, entry);
        previous = entry;
    }
    let outline = f.read();
    assert!(outline.partial);
    assert_eq!(outline.entries.len(), 8);
    assert!(
        outline
            .entries
            .iter()
            .all(|e| e.page.is_none() && e.issue == Some(PdfOutlineIssue::Unavailable))
    );
}

#[test]
fn external_and_script_actions_never_follow_local_looking_targets_or_next() {
    let mut f = Fixture::new();
    let mut previous = f.outlines;
    for (index, kind) in [
        "GoToR",
        "GoToE",
        "URI",
        "Launch",
        "JavaScript",
        "SetOCGState",
    ]
    .into_iter()
    .enumerate()
    {
        let entry = f.entry(kind, None);
        let action = dictionary! {
            "S" => kind, "D" => f.destination(0),
            "Next" => dictionary! { "S" => "GoTo", "D" => f.destination(1) },
        };
        f.document
            .get_dictionary_mut(entry)
            .unwrap()
            .set("A", action);
        f.link(previous, if index == 0 { "First" } else { "Next" }, entry);
        previous = entry;
    }
    let outline = f.read();
    assert!(!outline.partial);
    assert_eq!(outline.entries.len(), 6);
    assert!(
        outline
            .entries
            .iter()
            .all(|e| e.page.is_none() && e.issue == Some(PdfOutlineIssue::External))
    );
}

#[test]
fn linked_outline_reference_and_named_destination_cycles_are_bounded() {
    let mut f = Fixture::new();
    let first = f.entry("first", Some(f.destination(0)));
    let second = f.entry("second", Some(Object::Name(b"cycle".to_vec())));
    let refs = f.document.new_object_id();
    f.document.set_object(refs, Object::Reference(refs));
    let third = f.entry("indirect cycle", Some(refs.into()));
    let dests = dictionary! { "cycle" => Object::Name(b"cycle".to_vec()) };
    f.document
        .get_dictionary_mut(f.catalog)
        .unwrap()
        .set("Dests", dests);
    f.link(f.outlines, "First", first);
    f.link(first, "Next", second);
    f.link(second, "Next", third);
    f.link(third, "Next", first);
    let outline = f.read();
    assert!(outline.partial);
    assert_eq!(outline.entries.len(), 3);
    assert_eq!(outline.entries[0].page, Some(PhysicalPage(0)));
    assert!(
        outline.entries[1..]
            .iter()
            .all(|e| e.issue == Some(PdfOutlineIssue::Unavailable))
    );
}

#[test]
fn node_and_indirect_resolution_limits_return_partial_metadata() {
    let mut f = Fixture::new();
    let mut previous = f.outlines;
    for index in 0..MAX_NODES + 1 {
        let entry = f.entry("bounded", Some(f.destination(0)));
        f.link(previous, if index == 0 { "First" } else { "Next" }, entry);
        previous = entry;
    }
    let outline = f.read();
    assert_eq!(outline.entries.len(), MAX_ROWS);
    assert!(outline.partial);

    let mut f = Fixture::new();
    let mut target = f.destination(1);
    for _ in 0..MAX_HOPS + 1 {
        target = f.document.add_object(target).into();
    }
    let first = f.entry("too indirect", Some(target));
    f.link(f.outlines, "First", first);
    let outline = f.read();
    assert!(outline.partial);
    assert_eq!(outline.entries[0].issue, Some(PdfOutlineIssue::Unavailable));
}

#[test]
fn name_tree_cycles_odd_pairs_and_work_limits_preserve_good_rows() {
    let mut f = Fixture::new();
    let leaf = f.document.new_object_id();
    f.document.set_object(leaf, dictionary! {
        "Names" => vec![Object::string_literal("good"), f.destination(5), Object::string_literal("odd")],
        "Kids" => vec![Object::Reference(leaf)],
    });
    f.document
        .get_dictionary_mut(f.catalog)
        .unwrap()
        .set("Names", dictionary! { "Dests" => leaf });
    let first = f.entry("good", Some(Object::string_literal("good")));
    f.link(f.outlines, "First", first);
    let outline = f.read();
    assert!(outline.partial);
    assert_eq!(outline.entries[0].page, Some(PhysicalPage(5)));

    let mut f = Fixture::new();
    let names = (0..MAX_NODES + 1)
        .flat_map(|index| [Object::string_literal(index.to_string()), f.destination(0)])
        .collect::<Vec<_>>();
    f.document.get_dictionary_mut(f.catalog).unwrap().set(
        "Names",
        dictionary! {
            "Dests" => dictionary! { "Names" => names }
        },
    );
    let first = f.entry("first", Some(Object::string_literal("0")));
    f.link(f.outlines, "First", first);
    let outline = f.read();
    assert!(outline.partial);
    assert_eq!(outline.entries[0].page, Some(PhysicalPage(0)));
}

#[test]
fn absent_empty_and_unreadable_outline_do_not_change_page_loading() {
    let mut f = Fixture::new();
    assert_eq!(f.read(), PdfOutline::default());
    f.document
        .get_dictionary_mut(f.catalog)
        .unwrap()
        .remove(b"Outlines");
    assert_eq!(f.read(), PdfOutline::default());
    f.document
        .get_dictionary_mut(f.catalog)
        .unwrap()
        .set("Outlines", Object::Null);
    assert_eq!(
        f.read(),
        PdfOutline {
            entries: vec![],
            partial: true
        }
    );
}
