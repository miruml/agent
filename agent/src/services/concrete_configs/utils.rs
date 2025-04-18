use crate::storage;

pub fn convert_cncr_cfg_backend_to_storage(
    backend_concrete_config: openapi_client::models::BackendConcreteConfig,
    config_slug: String,
    config_schema_digest: String,
) -> storage::concrete_configs::ConcreteConfig {
    storage::concrete_configs::ConcreteConfig {
        id: backend_concrete_config.id,
        created_at: backend_concrete_config.created_at.unwrap_or_default(),
        client_id: backend_concrete_config.client_id,
        config_schema_id: backend_concrete_config.config_schema_id,
        concrete_config: backend_concrete_config.concrete_config.unwrap_or_default(),
        config_slug,
        config_schema_digest,
    }
}

pub fn convert_cncr_cfg_storage_to_sdk(
    storage_concrete_config: storage::concrete_configs::ConcreteConfig,
) -> openapi_server::models::BaseConcreteConfig {
    openapi_server::models::BaseConcreteConfig {
        object: openapi_server::models::base_concrete_config::Object::ConcreteConfig,
        id: storage_concrete_config.id,
        created_at: Some(storage_concrete_config.created_at),
        client_id: storage_concrete_config.client_id,
        config_schema_id: storage_concrete_config.config_schema_id,
        concrete_config: Some(storage_concrete_config.concrete_config),
    }
}
