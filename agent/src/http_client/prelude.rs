// internal crates
pub use crate::http_client::concrete_configs::ConcreteConfigsExt;
pub use crate::http_client::config_schemas::ConfigSchemasExt;

pub trait HTTPClientExt: ConcreteConfigsExt + ConfigSchemasExt {}