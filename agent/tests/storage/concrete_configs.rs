#[cfg(test)]
mod tests {
    // std
    use std::path::PathBuf;

    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    use config_agent::storage::{
        concrete_configs::ConcreteConfigCache,
        errors::StorageErr,
    };
    use openapi_client::models::BackendConcreteConfig;

    // external crates
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod new {
    use super::*;

    #[tokio::test]
    async fn spawn() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let _ = ConcreteConfigCache::spawn(dir.clone());
        // the directory should not exist yet
        assert!(!dir.exists());
    }
}

pub mod read {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        assert!(matches!(
            cache.read("config_slug", "config_schema_digest").await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        let concrete_config = BackendConcreteConfig::default();
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), false).await.unwrap();

        // reading the concrete config should return the concrete config
        let read_concrete_config = cache.read("config_slug", "config_schema_digest").await.unwrap();
        assert_eq!(read_concrete_config, concrete_config);
    }
}

pub mod read_optional {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        assert!(matches!(
            cache.read("config_slug", "config_schema_digest").await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        let concrete_config = BackendConcreteConfig::default();
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), false).await.unwrap();

        // reading the concrete config should return the concrete config
        let read_concrete_config = cache.read_optional("config_slug", "config_schema_digest").await.unwrap().unwrap();
        assert_eq!(read_concrete_config, concrete_config);
    }
}

pub mod write {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        let concrete_config = BackendConcreteConfig::default();
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), false).await.unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the concrete config should return the concrete config
        let read_concrete_config = cache.read_optional("config_slug", "config_schema_digest").await.unwrap().unwrap();
        assert_eq!(read_concrete_config, concrete_config);
    }

    #[tokio::test]
    async fn doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        let concrete_config = BackendConcreteConfig::default();

        // writing the concrete config should overwrite the existing concrete config
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), true).await.unwrap();

        // reading the concrete config should return the concrete config
        let read_concrete_config = cache.read_optional("config_slug", "config_schema_digest").await.unwrap().unwrap();
        assert_eq!(read_concrete_config, concrete_config);
    }

    #[tokio::test]
    async fn exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        let concrete_config = BackendConcreteConfig::default();
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), false).await.unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache.write("config_slug", "config_schema_digest", concrete_config.clone(), false).await.unwrap_err(),
            StorageErr::FileSysErr { .. }
        ));
    }

    #[tokio::test]
    async fn exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("concrete_cfg_cache"));
        let cache = ConcreteConfigCache::spawn(dir.clone());
        let concrete_config = BackendConcreteConfig::default();
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), true).await.unwrap();

        // should not throw an error since overwrite is true
        cache.write("config_slug", "config_schema_digest", concrete_config.clone(), true).await.unwrap();

        // reading the concrete config should return the concrete config
        let read_concrete_config = cache.read_optional("config_slug", "config_schema_digest").await.unwrap().unwrap();
        assert_eq!(read_concrete_config, concrete_config);
    }
}
}