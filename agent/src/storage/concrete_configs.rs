// internal crates
use crate::filesys::{
    dir::Dir,
    file,
    file::File,
    path::PathExt,
};
use crate::storage::errors::StorageErr;
use crate::trace;
use openapi_client::models::BackendConcreteConfig;

pub struct LatestConcreteConfigRegistry {
    pub dir: Dir,
}

impl LatestConcreteConfigRegistry {
    pub fn new(dir: Dir) -> Self {
        Self { 
            dir, 
        }
    }

    fn config_schema_file(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> File {
        let filename = format!(
            "{}_{}.json",
            config_slug,
            config_schema_digest,
        );
        let filename = file::sanitize_filename(&filename);
        self.dir.file(&filename)
    }

    pub fn read(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<BackendConcreteConfig>, StorageErr> {
        let config_file = self.config_schema_file(config_slug, config_schema_digest);
        if !config_file.exists() {
            return Ok(None);
        }

        let config = config_file.read_json::<BackendConcreteConfig>().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(config))
    }

    pub fn insert(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
        config: &BackendConcreteConfig,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let config_file = self.config_schema_file(config_slug, config_schema_digest);
        config_file.write_json(config, overwrite).map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(())
    }
}