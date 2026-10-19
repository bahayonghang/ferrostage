use ferrostage_core::{PhysicalPage, SlideMap};
use ferrostage_pdf::{PdfOutline, PdfOutlineIssue};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutlineEntry {
    pub id: usize,
    pub parent: Option<usize>,
    pub depth: u8,
    pub title: Option<String>,
    pub page: Option<PhysicalPage>,
    pub content_number: Option<usize>,
    pub issue: Option<OutlineIssue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutlineIssue {
    Notes,
    Unavailable,
    External,
}

pub(crate) fn map(
    outline: &PdfOutline,
    slide_map: &SlideMap,
    labels: &[Option<String>],
) -> Vec<OutlineEntry> {
    outline
        .entries
        .iter()
        .map(|entry| {
            let content_number = entry
                .page
                .and_then(|page| {
                    slide_map
                        .steps()
                        .binary_search_by_key(&page.0, |p| p.0)
                        .ok()
                })
                .map(|index| index + 1);
            let issue = match entry.issue {
                Some(PdfOutlineIssue::External) => Some(OutlineIssue::External),
                Some(PdfOutlineIssue::Unavailable) => Some(OutlineIssue::Unavailable),
                None if content_number.is_some() => None,
                None => Some(
                    if entry.page.is_some_and(|page| {
                        labels
                            .get(page.0)
                            .and_then(Option::as_deref)
                            .is_some_and(|label| label.starts_with("notes:"))
                    }) {
                        OutlineIssue::Notes
                    } else {
                        OutlineIssue::Unavailable
                    },
                ),
            };
            OutlineEntry {
                id: entry.id,
                parent: entry.parent,
                depth: entry.depth,
                title: entry.title.clone(),
                page: entry.page,
                content_number: content_number.filter(|_| issue.is_none()),
                issue,
            }
        })
        .collect()
}
