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

// external crates
use serde::Deserialize;
use serde::Serialize;

pub struct ConcreteConfigRegistry {
    pub dir: Dir,
    id_registry: ConcreteConfigIDRegistry,
}

impl ConcreteConfigRegistry {
    pub fn new(dir: Dir) -> Self {
        let id_registry = ConcreteConfigIDRegistry::new(
            dir.subdir("id_registry")
        );
        Self { 
            dir, 
            id_registry,
        }
    }

    fn concrete_config_file(&self, cncr_cfg_id: &str) -> File {
        let mut filename = format!("{}.json", cncr_cfg_id);
        filename = file::sanitize_filename(&filename);
        self.dir.file(&filename)
    }

    pub fn get_concrete_config_id(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<String>, StorageErr> {
        self.id_registry.read(config_slug, config_schema_digest)
    }

    pub fn read(
        &self,
        cncr_cfg_id: &str,
    ) -> Result<Option<BackendConcreteConfig>, StorageErr> {
        let config_file = self.concrete_config_file(cncr_cfg_id);
        if !config_file.exists() {
            return Ok(None);
        }

        let config = config_file.read_json::<BackendConcreteConfig>().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(config))
    }

}

#[derive(Debug, Serialize, Deserialize)]
struct ConcreteConfigID {
    #[serde(rename = "concrete_config_id")]
    pub id: String,
}

struct ConcreteConfigIDRegistry {
    pub dir: Dir,
}

impl ConcreteConfigIDRegistry {
    pub fn new(dir: Dir) -> Self {
        Self { dir }
    }

    fn id_file(
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

    fn read(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
    ) -> Result<Option<String>, StorageErr> {
        let id_file = self.id_file(config_slug, config_schema_digest);
        if !id_file.exists() {
            return Ok(None);
        }

        let id = id_file.read_json::<ConcreteConfigID>().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(id.id))
    }

    fn insert(
        &self,
        config_slug: &str,
        config_schema_digest: &str,
        id: &str,
    ) -> Result<(), StorageErr> {
        let id_file = self.id_file(config_slug, config_schema_digest);
        let concrete_config_id = ConcreteConfigID { id: id.to_string() };
        id_file.write_json(
            &concrete_config_id,
            false,
        ).map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(())
    }
}

