#[cfg(test)]
mod tests {
    // std
    use std::path::PathBuf;

    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    use config_agent::storage::{
        cfg_sch_digest_reg::{
            AsyncConfigSchemaDigestCache,
            ConfigSchemaDigests,
            SyncConfigSchemaDigestCache,
        },
        errors::StorageErr,
    };

    // external crates
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod sync {
    use super::*;

pub mod new {
    use super::*;

    #[test]
    fn not_created() {
        let dir = Dir::create_temp_dir("testing").unwrap().subdir(PathBuf::from("cfg_sch_digest_reg"));
        let _ = SyncConfigSchemaDigestCache::new(dir.clone());
        // the directory should not exist yet
        assert!(!dir.exists());
    }
}

pub mod read {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        // synchronous cache
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        assert!(matches!(
            cache.read("1234567890").unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).unwrap();

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").unwrap();
        assert_eq!(read_digests, digests);
    }
}

pub mod read_optional {
    use super::*;

    #[test]
    fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let read_digests = cache.read_optional("1234567890").unwrap();
        assert_eq!(read_digests, None);
    }

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).unwrap();
        let read_digests = cache.read_optional("1234567890").unwrap().unwrap();
        assert_eq!(read_digests, digests);
    }
}

pub mod insert {
    use super::*;

    #[test]
    fn doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").unwrap();
        assert_eq!(read_digests, digests);
    }

    #[test]
    fn doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), true).unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").unwrap();
        assert_eq!(read_digests, digests);
    }

    #[test]
    fn exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache.insert(digests.clone(), false).unwrap_err(),
            StorageErr::FileSysErr { .. }
        ));
    }

    #[test]
    fn exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = SyncConfigSchemaDigestCache::new(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), true).unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").unwrap();
        assert_eq!(read_digests, digests);
    }
}
}

pub mod async_ {
    use super::*;

pub mod new {
    use super::*;

    #[tokio::test]
    async fn spawn() {
        let dir = Dir::create_temp_dir("testing").unwrap().subdir(PathBuf::from("cfg_sch_digest_reg"));
        let _ = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        // the directory should not exist yet
        assert!(!dir.exists());
    }
}

pub mod read {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        assert!(matches!(
            cache.read("1234567890").await.unwrap_err(),
            StorageErr::CacheElementNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).await.unwrap();

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }
}

pub mod read_optional {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let read_digests = cache.read_optional("1234567890").await.unwrap();
        assert_eq!(read_digests, None);
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).await.unwrap();
        let read_digests = cache.read_optional("1234567890").await.unwrap().unwrap();
        assert_eq!(read_digests, digests);
    }
}

pub mod insert {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).await.unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }

    #[tokio::test]
    async fn doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), true).await.unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }

    #[tokio::test]
    async fn exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), false).await.unwrap();

        // should throw an error since already exists
        assert!(matches!(
            cache.insert(digests.clone(), false).await.unwrap_err(),
            StorageErr::FileSysErr { .. }
        ));
    }

    #[tokio::test]
    async fn exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let cache = AsyncConfigSchemaDigestCache::spawn(dir.clone());
        let digests = ConfigSchemaDigests {
            raw: "1234567890".to_string(),
            resolved: "1234567890".to_string(),
        };
        cache.insert(digests.clone(), true).await.unwrap();

        // the directory should exist now
        assert!(dir.exists());

        // reading the digests should return the digests
        let read_digests = cache.read("1234567890").await.unwrap();
        assert_eq!(read_digests, digests);
    }
}
}
}