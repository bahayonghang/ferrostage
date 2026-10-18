use crate::LoadedDocument;
use ferrostage_core::PhysicalPage;

pub struct ReloadMapping {
    pub position: PhysicalPage,
    pub pages: Vec<(PhysicalPage, PhysicalPage)>,
}
impl LoadedDocument {
    /// Unique logical label plus overlay ordinal; ambiguous labels use physical
    /// coordinates. An absent annotation page is never clamped onto another page.
    pub fn reload_mapping(&self, next: &Self, current: PhysicalPage) -> ReloadMapping {
        let labelled = |page: PhysicalPage| {
            let old_group = self.slide_map.logical_for(page)?;
            let group = &self.slide_map.groups()[old_group.0];
            let label = self.labels[group[0].0]
                .as_deref()
                .filter(|s| !s.is_empty())?;
            let unique = |document: &Self| {
                let mut groups = document
                    .slide_map
                    .groups()
                    .iter()
                    .enumerate()
                    .filter(|(_, group)| document.labels[group[0].0].as_deref() == Some(label));
                let found = groups.next()?;
                groups.next().is_none().then_some(found.0)
            };
            unique(self)?;
            let target = &next.slide_map.groups()[unique(next)?];
            let ordinal = group.iter().position(|p| *p == page)?;
            target.get(ordinal).copied()
        };
        let position = labelled(current).unwrap_or_else(|| {
            next.slide_map
                .closest_content_at_or_before(PhysicalPage(current.0.min(next.page_count - 1)))
        });
        let mut pages = Vec::new();
        for &old in self.slide_map.steps() {
            let new =
                labelled(old).or_else(|| next.slide_map.steps().contains(&old).then_some(old));
            if let Some(new) = new
                && !pages.iter().any(|(_, used)| *used == new)
            {
                pages.push((old, new));
            }
        }
        ReloadMapping { position, pages }
    }
}
