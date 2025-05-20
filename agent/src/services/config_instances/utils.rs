use crate::storage;

pub fn convert_cfg_inst_backend_to_storage(
    backend_config_instance: openapi_client::models::BackendConfigInstance,
    config_slug: String,
    config_schema_digest: String,
) -> storage::config_instances::ConfigInstance {
    storage::config_instances::ConfigInstance {
        id: backend_config_instance.id,
        created_at: backend_config_instance.created_at,
        client_id: backend_config_instance.device_id,
        config_schema_id: backend_config_instance.config_schema_id,
        config_instance: backend_config_instance.config_instance.unwrap_or_default(),
        config_slug,
        config_schema_digest,
    }
}

pub fn convert_cfg_inst_storage_to_sdk(
    storage_config_instance: storage::config_instances::ConfigInstance,
) -> openapi_server::models::BaseConfigInstance {
    openapi_server::models::BaseConfigInstance {
        object: openapi_server::models::base_config_instance::Object::ConfigInstance,
        id: storage_config_instance.id,
        created_at: storage_config_instance.created_at,
        device_id: storage_config_instance.client_id,
        config_schema_id: storage_config_instance.config_schema_id,
        config_instance: Some(storage_config_instance.config_instance),
    }
}
