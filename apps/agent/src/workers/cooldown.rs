#[derive(Debug, Clone, Copy)]
pub struct CooldownOptions {
    pub base_secs: i64,
    pub growth_factor: i64,
    pub max_secs: i64,
}

impl Default for CooldownOptions {
    fn default() -> Self {
        Self {
            base_secs: 15,
            growth_factor: 2,
            max_secs: 12 * 60 * 60, // 12 hours
        }
    }
}