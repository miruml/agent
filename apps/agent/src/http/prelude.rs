// internal crates
pub use crate::http::config_instances::ConfigInstancesExt;
pub use crate::http::config_schemas::ConfigSchemasExt;

pub trait HTTPClientExt: ConfigInstancesExt + ConfigSchemasExt {}
