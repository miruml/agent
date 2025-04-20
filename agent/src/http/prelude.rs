// internal crates
pub use crate::http::concrete_configs::ConcreteConfigsExt;
pub use crate::http::config_schemas::ConfigSchemasExt;

pub trait HTTPClientExt: ConcreteConfigsExt + ConfigSchemasExt {}
