//! Bounded, presenter-only bookmark metadata. No PDF action is executed.
use ferrostage_core::PhysicalPage;
use lopdf::{Dictionary, Document, Object, ObjectId};
use std::collections::{BTreeMap, BTreeSet};

const MAX_NODES: usize = 4096;
const MAX_ROWS: usize = 4096;
const MAX_HOPS: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PdfOutline {
    pub entries: Vec<PdfOutlineEntry>,
    pub partial: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfOutlineEntry {
    pub id: usize,
    pub parent: Option<usize>,
    pub depth: u8,
    pub title: Option<String>,
    pub page: Option<PhysicalPage>,
    pub issue: Option<PdfOutlineIssue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfOutlineIssue {
    Unavailable,
    External,
}

pub(super) fn extract(document: &Document) -> PdfOutline {
    let mut parser = OutlineParser {
        document,
        pages: document
            .get_pages()
            .into_values()
            .enumerate()
            .map(|(index, id)| (id, PhysicalPage(index)))
            .collect(),
        names: BTreeMap::new(),
        result: PdfOutline::default(),
    };
    let Some(catalog) = document
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|root| parser.dict(root))
    else {
        parser.result.partial = true;
        return parser.result;
    };
    let Ok(outlines) = catalog.get(b"Outlines") else {
        return parser.result;
    };
    let Some(outlines) = parser.dict(outlines) else {
        parser.result.partial = true;
        return parser.result;
    };
    parser.read_names(catalog);
    if let Ok(first) = outlines.get(b"First") {
        parser.walk(first);
    }
    parser.result
}

struct OutlineParser<'a> {
    document: &'a Document,
    pages: BTreeMap<ObjectId, PhysicalPage>,
    names: BTreeMap<Vec<u8>, &'a Object>,
    result: PdfOutline,
}

impl<'a> OutlineParser<'a> {
    /// Unlike unbounded linked-list traversal, every indirect value has a small ceiling.
    fn resolve(&self, mut object: &'a Object) -> Option<(Option<ObjectId>, &'a Object)> {
        let mut identity = None;
        for _ in 0..MAX_HOPS {
            match object {
                Object::Reference(id) => {
                    identity = Some(*id);
                    object = self.document.objects.get(id)?;
                }
                _ => return Some((identity, object)),
            }
        }
        (!matches!(object, Object::Reference(_))).then_some((identity, object))
    }

    fn dict(&self, object: &'a Object) -> Option<&'a Dictionary> {
        self.resolve(object)?.1.as_dict().ok()
    }

    fn walk(&mut self, first: &'a Object) {
        let mut pending = vec![(first, None, 1)];
        let mut visited = BTreeSet::new();
        let mut nodes = 0;
        while let Some((object, parent, depth)) = pending.pop() {
            if nodes == MAX_NODES || self.result.entries.len() == MAX_ROWS {
                self.result.partial = true;
                break;
            }
            nodes += 1;
            let Some((identity, object)) = self.resolve(object) else {
                self.result.partial = true;
                continue;
            };
            if identity.is_some_and(|id| !visited.insert(id)) {
                self.result.partial = true;
                continue;
            }
            let Ok(dict) = object.as_dict() else {
                self.result.partial = true;
                continue;
            };
            let title = dict
                .get(b"Title")
                .ok()
                .and_then(|title| self.resolve(title))
                .and_then(|(_, title)| lopdf::decode_text_string(title).ok())
                .filter(|title| !title.trim().is_empty());
            if title.is_none() {
                self.result.partial = true;
            }
            let (page, issue) = self.target(dict);
            let id = self.result.entries.len();
            self.result.entries.push(PdfOutlineEntry {
                id,
                parent,
                depth,
                title,
                page,
                issue,
            });
            if let Ok(next) = dict.get(b"Next") {
                pending.push((next, parent, depth));
            }
            // Intentionally do not even inspect a third-level subtree.
            if depth == 1
                && let Ok(child) = dict.get(b"First")
            {
                pending.push((child, Some(id), 2));
            }
        }
    }

    fn target(&mut self, dict: &'a Dictionary) -> (Option<PhysicalPage>, Option<PdfOutlineIssue>) {
        let destination = if let Ok(destination) = dict.get(b"Dest") {
            Some(destination)
        } else if let Ok(action) = dict.get(b"A") {
            let Some(action) = self.dict(action) else {
                return self.unavailable();
            };
            let kind = action
                .get(b"S")
                .ok()
                .and_then(|kind| self.resolve(kind))
                .and_then(|(_, kind)| kind.as_name().ok());
            match kind {
                Some(b"GoTo") => action.get(b"D").ok(),
                Some(_) => return (None, Some(PdfOutlineIssue::External)),
                None => return self.unavailable(),
            }
        } else {
            None
        };
        match destination.and_then(|destination| self.destination(destination)) {
            Some(page) => (Some(page), None),
            None => self.unavailable(),
        }
    }

    fn unavailable(&mut self) -> (Option<PhysicalPage>, Option<PdfOutlineIssue>) {
        self.result.partial = true;
        (None, Some(PdfOutlineIssue::Unavailable))
    }

    fn destination(&self, mut destination: &'a Object) -> Option<PhysicalPage> {
        let mut names = BTreeSet::new();
        for _ in 0..MAX_HOPS {
            let (_, resolved) = self.resolve(destination)?;
            match resolved {
                Object::Array(array) => {
                    let mode = self.resolve(array.get(1)?)?.1.as_name().ok()?;
                    let minimum = match mode {
                        b"Fit" | b"FitB" => 2,
                        b"FitH" | b"FitV" | b"FitBH" | b"FitBV" => 3,
                        b"XYZ" => 5,
                        b"FitR" => 6,
                        _ => return None,
                    };
                    if array.len() < minimum {
                        return None;
                    }
                    let (identity, _) = self.resolve(array.first()?)?;
                    return self.pages.get(&identity?).copied();
                }
                Object::Name(name) | Object::String(name, _) => {
                    if !names.insert(name) {
                        return None;
                    }
                    destination = self.names.get(name)?;
                }
                Object::Dictionary(dict) => destination = dict.get(b"D").ok()?,
                _ => return None,
            }
        }
        None
    }

    fn read_names(&mut self, catalog: &'a Dictionary) {
        if let Ok(dests) = catalog.get(b"Dests") {
            if let Some(dests) = self.dict(dests) {
                for (index, (name, destination)) in dests.iter().enumerate() {
                    if index == MAX_NODES {
                        self.result.partial = true;
                        break;
                    }
                    self.names.insert(name.clone(), destination);
                }
            } else {
                self.result.partial = true;
            }
        }
        let Some(root) = catalog
            .get(b"Names")
            .ok()
            .and_then(|names| self.dict(names))
            .and_then(|names| names.get(b"Dests").ok())
        else {
            return;
        };
        let mut pending = vec![root];
        let mut visited = BTreeSet::new();
        let mut nodes = 0;
        let mut pairs = 0;
        while let Some(object) = pending.pop() {
            if nodes == MAX_NODES {
                self.result.partial = true;
                break;
            }
            nodes += 1;
            let Some((identity, object)) = self.resolve(object) else {
                self.result.partial = true;
                continue;
            };
            if identity.is_some_and(|id| !visited.insert(id)) {
                self.result.partial = true;
                continue;
            }
            let Ok(dict) = object.as_dict() else {
                self.result.partial = true;
                continue;
            };
            if let Ok(names) = dict.get(b"Names") {
                if let Some((_, Object::Array(names))) = self.resolve(names) {
                    if names.len() % 2 != 0 {
                        self.result.partial = true;
                    }
                    for pair in names.as_chunks::<2>().0 {
                        if pairs == MAX_NODES {
                            self.result.partial = true;
                            return;
                        }
                        pairs += 1;
                        match self.resolve(&pair[0]).map(|(_, name)| name) {
                            Some(Object::String(name, _) | Object::Name(name)) => {
                                self.names.entry(name.clone()).or_insert(&pair[1]);
                            }
                            _ => self.result.partial = true,
                        }
                    }
                } else {
                    self.result.partial = true;
                }
            }
            if let Ok(kids) = dict.get(b"Kids") {
                if let Some((_, Object::Array(kids))) = self.resolve(kids) {
                    // Bound the frontier as well as visited nodes for very wide trees.
                    let remaining = MAX_NODES.saturating_sub(nodes + pending.len());
                    if kids.len() > remaining {
                        self.result.partial = true;
                    }
                    pending.extend(kids.iter().take(remaining).rev());
                } else {
                    self.result.partial = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
