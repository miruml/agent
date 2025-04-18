#[cfg(test)]
mod tests {
    // std
    use std::path::PathBuf;

    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    use config_agent::storage::{
        digests::{
            ConfigSchemaDigestCache,
            ConfigSchemaDigests,
        },
        errors::StorageErr,
    };

    // external crates
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};


pub mod new {
    use super::*;

    #[tokio::test]
    async fn spawn() {
        let dir = Dir::create_temp_dir("testing").await.unwrap().subdir(PathBuf::from("cfg_sch_digest_reg"));
        let _ = ConfigSchemaDigestCache::spawn(dir.clone());
        // the directory should not exist yet
        assert!(!dir.exists());
    }
}

pub mod read {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        assert!(matches!(
            cache.read("1234567890").await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.write(digests.clone(), false).await.unwrap();

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }
}

pub mod read_optional {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let read_digests = cache.read_optional("1234567890").await.unwrap();
        assert_eq!(read_digests, None);
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.write(digests.clone(), false).await.unwrap();
        let read_digests = cache.read_optional("1234567890").await.unwrap().unwrap();
        assert_eq!(read_digests, digests);
    }
}

pub mod write {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.write(digests.clone(), false).await.unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }

    #[tokio::test]
    async fn doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.write(digests.clone(), true).await.unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }

    #[tokio::test]
    async fn exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.write(digests.clone(), false).await.unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache.write(digests.clone(), false).await.unwrap_err(),
            StorageErr::FileSysErr { .. }
        ));
    }

    #[tokio::test]
    async fn exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let cache = ConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.write(digests.clone(), false).await.unwrap();

        // should not throw an error since overwrite is true
        cache.write(digests.clone(), true).await.unwrap();

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }
}
}