use crate::{CoreError, LogicalSlide, PhysicalPage};

/// Validated navigation over content pages only. Labels and notes belong to document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlideMap {
    page_count: usize,
    groups: Vec<Vec<PhysicalPage>>,
    steps: Vec<PhysicalPage>,
}

impl SlideMap {
    pub fn new(page_count: usize, groups: Vec<Vec<PhysicalPage>>) -> Result<Self, CoreError> {
        if page_count == 0 {
            return Err(CoreError::EmptyDocument);
        }
        if groups.is_empty() {
            return Err(CoreError::EmptyContent);
        }
        let mut steps: Vec<PhysicalPage> = Vec::new();
        for group in &groups {
            if group.is_empty() {
                return Err(CoreError::InvalidSlideMap);
            }
            for &page in group {
                if page.0 >= page_count || steps.last().is_some_and(|last| last.0 >= page.0) {
                    return Err(CoreError::InvalidSlideMap);
                }
                steps.push(page);
            }
        }
        Ok(Self {
            page_count,
            groups,
            steps,
        })
    }

    pub fn identity(page_count: usize) -> Result<Self, CoreError> {
        Self::new(
            page_count,
            (0..page_count).map(|p| vec![PhysicalPage(p)]).collect(),
        )
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }
    pub fn steps(&self) -> &[PhysicalPage] {
        &self.steps
    }
    pub fn groups(&self) -> &[Vec<PhysicalPage>] {
        &self.groups
    }
    pub fn logical_for(&self, page: PhysicalPage) -> Option<LogicalSlide> {
        self.groups
            .iter()
            .position(|g| g.contains(&page))
            .map(LogicalSlide)
    }
    pub fn first_step(&self, slide: LogicalSlide) -> Option<PhysicalPage> {
        self.groups.get(slide.0).map(|g| g[0])
    }
    pub fn closest_content_at_or_before(&self, page: PhysicalPage) -> PhysicalPage {
        let index = self
            .steps
            .partition_point(|p| p.0 <= page.0)
            .saturating_sub(1);
        self.steps[index]
    }
    pub fn next_step(&self, page: PhysicalPage) -> Option<PhysicalPage> {
        self.step(page, 1)
    }
    pub fn previous_step(&self, page: PhysicalPage) -> Option<PhysicalPage> {
        self.step(page, -1)
    }
    fn step(&self, page: PhysicalPage, delta: isize) -> Option<PhysicalPage> {
        let index = self.steps.binary_search_by_key(&page.0, |p| p.0).ok()?;
        Some(self.steps[index.saturating_add_signed(delta).min(self.steps.len() - 1)])
    }
    pub fn next_logical(&self, page: PhysicalPage) -> Option<PhysicalPage> {
        self.logical(page, 1)
    }
    pub fn previous_logical(&self, page: PhysicalPage) -> Option<PhysicalPage> {
        self.logical(page, -1)
    }
    fn logical(&self, page: PhysicalPage, delta: isize) -> Option<PhysicalPage> {
        let index = self.logical_for(page)?.0;
        self.first_step(LogicalSlide(
            index
                .saturating_add_signed(delta)
                .min(self.groups.len() - 1),
        ))
    }
}

#[cfg(test)]
mod tests;
