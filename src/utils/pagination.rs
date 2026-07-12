/// Validates and normalizes pagination parameters.
#[derive(Debug, Clone, Copy)]
pub struct PaginationParams {
    pub page: u32,
    pub per_page: u32,
}

impl PaginationParams {
    #[must_use]
    pub fn new(page: Option<u32>, per_page: Option<u32>) -> Self {
        Self { page: page.unwrap_or(1).max(1), per_page: per_page.unwrap_or(20).clamp(1, 100) }
    }

    #[must_use]
    pub fn offset(&self) -> u64 {
        u64::from((self.page - 1) * self.per_page)
    }

    #[must_use]
    pub fn limit(&self) -> i64 {
        i64::from(self.per_page)
    }
}

#[must_use]
pub fn total_pages(total: u64, per_page: u32) -> u32 {
    total.div_ceil(u64::from(per_page)) as u32
}
