// standard library
use std::future::Future;
use std::sync::Arc;

// internal crates
use crate::storage::config_instances::{ConfigInstanceCache, ConfigInstanceDataCache};
use crate::storage::config_schemas::ConfigSchemaCache;
use crate::storage::digests::ConfigSchemaDigestCache;
use crate::storage::layout::StorageLayout;
use crate::storage::errors::*;
use crate::trace;

#[derive(Copy, Clone, Debug)]
pub struct CacheSizes {
    pub cfg_sch_digest: usize,
    pub cfg_inst_metadata: usize,
    pub cfg_inst_data: usize,
    pub cfg_schema: usize,
}

impl Default for CacheSizes {
    fn default() -> Self {
        Self {
            cfg_sch_digest: 100,
            cfg_inst_metadata: 100,
            cfg_inst_data: 100,
            cfg_schema: 100,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Caches {
    pub cfg_sch_digest: Arc<ConfigSchemaDigestCache>,
    pub cfg_inst_metadata: Arc<ConfigInstanceCache>,
    pub cfg_inst_data: Arc<ConfigInstanceDataCache>,
    pub cfg_schema: Arc<ConfigSchemaCache>,
    pub sizes: CacheSizes,
}

impl Caches {
    pub async fn init(
        layout: &StorageLayout,
        sizes: CacheSizes,
    ) -> Result<(Caches, impl Future<Output = ()>), StorageErr> {

        // config schema digests
        let (cfg_sch_digest_cache, cfg_sch_digest_cache_handle) =
            ConfigSchemaDigestCache::spawn(
                64,
                layout.config_schema_digest_cache(),
                sizes.cfg_sch_digest,
            )
                .await
                .map_err(|e| {
                    StorageErr::CacheErr(Box::new(StorageCacheErr {
                        source: e,
                        trace: trace!(),
                    }))
                })?;
        let cfg_sch_digest_cache = Arc::new(cfg_sch_digest_cache);

        // config schemas
        let (cfg_schema_cache, cfg_schema_cache_handle) =
            ConfigSchemaCache::spawn(
                64,
                layout.config_schema_cache(),
                sizes.cfg_schema,
            )
                .await
                .map_err(|e| {
                    StorageErr::CacheErr(Box::new(StorageCacheErr {
                        source: e,
                        trace: trace!(),
                    }))
                })?;
        let cfg_schema_cache = Arc::new(cfg_schema_cache);

        // config instance metadata
        let (cfg_inst_metadata_cache, cfg_inst_metadata_cache_handle) =
            ConfigInstanceCache::spawn(
                64,
                layout.config_instance_metadata_cache(),
                sizes.cfg_inst_metadata,
            )
                .await
                .map_err(|e| {
                    StorageErr::CacheErr(Box::new(StorageCacheErr {
                        source: e,
                        trace: trace!(),
                    }))
                })?;
        let cfg_inst_metadata_cache = Arc::new(cfg_inst_metadata_cache);

        // config instance data
        let (cfg_inst_data_cache, cfg_inst_data_cache_handle) =
            ConfigInstanceDataCache::spawn(
                64,
                layout.config_instance_data_cache(),
                sizes.cfg_inst_data,
            )
                .await
                .map_err(|e| {
                    StorageErr::CacheErr(Box::new(StorageCacheErr {
                        source: e,
                        trace: trace!(),
                    }))
                })?;
        let cfg_inst_data_cache = Arc::new(cfg_inst_data_cache);

        // return the shutdown handler
        let shutdown_handle = async move {
            let handles = vec![
                cfg_sch_digest_cache_handle,
                cfg_inst_metadata_cache_handle,
                cfg_inst_data_cache_handle,
                cfg_schema_cache_handle,
            ];

            futures::future::join_all(handles).await;
        };

        Ok((Caches {
            cfg_sch_digest: cfg_sch_digest_cache,
            cfg_inst_metadata: cfg_inst_metadata_cache,
            cfg_inst_data: cfg_inst_data_cache,
            cfg_schema: cfg_schema_cache,
            sizes,
        }, shutdown_handle))
    }

    pub async fn shutdown(&self) -> Result<(), StorageErr> {

        self.cfg_sch_digest.shutdown().await.map_err(|e| {
            StorageErr::CacheErr(Box::new(StorageCacheErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        self.cfg_inst_metadata.shutdown().await.map_err(|e| {
            StorageErr::CacheErr(Box::new(StorageCacheErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        self.cfg_inst_data.shutdown().await.map_err(|e| {
            StorageErr::CacheErr(Box::new(StorageCacheErr {
                source: e,
                trace: trace!(),
            }))
        })?;
        self.cfg_schema.shutdown().await.map_err(|e| {
            StorageErr::CacheErr(Box::new(StorageCacheErr {
                source: e,
                trace: trace!(),
            }))
        })?;

        Ok(())
    }
}