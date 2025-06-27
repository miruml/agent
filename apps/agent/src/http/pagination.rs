pub const MAX_PAGINATE_LIMIT: usize = 100;

pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { limit: 10, offset: 0 }
    }
}