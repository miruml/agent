// internal crates
use crate::filesys::dir::Dir;
use crate::filesys::file::File;
use crate::storage::errors::StorageErr;
use crate::storage::cfg_sch_digest_reg::ConfigSchemaDigestRegistry;

// external crates
use std::path::PathBuf;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct StorageLayout {
    pub root: Dir,
}

impl StorageLayout {

    pub fn new(root: Dir) -> Result<Self, StorageErr> {
        Ok(Self { root })
    }

    pub fn default_root_dir() -> Dir {
        Dir::new(PathBuf::from("var").join("lib").join("miru"))
    }

    pub fn new_default() -> Result<Self, StorageErr> {
        Self::new(Self::default_root_dir())
    }

    pub fn agent_cfg(&self) -> File {
        self.root.file("agent_config.json")
    }

    pub fn cfg_sch_digest_registry(&self) -> ConfigSchemaDigestRegistry {
        ConfigSchemaDigestRegistry::new(self.root.subdir("config_schema_digest_cache"))
    }

    pub fn concrete_config_cache_index(&self) -> File {
        self.root.file("concrete_config_cache_index.json")
    }

    pub fn concrete_configs(&self) -> Dir {
        self.root.subdir("concrete_configs")
    }
}
