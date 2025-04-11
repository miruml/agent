// internal crates
use crate::models::device_cfg::Timeouts;

pub struct HTTPArgs<'a> {
    pub token: &'a str,
    pub timeouts: &'a Timeouts,
}
