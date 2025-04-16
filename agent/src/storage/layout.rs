// internal crates
use crate::filesys::dir::Dir;
use crate::filesys::file::File;
use crate::storage::cfg_sch_digest_reg::ConfigSchemaDigestRegistry;
use crate::storage::cncr_cfg_reg::LatestConcreteConfigRegistry;

// external crates
use std::path::PathBuf;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct StorageLayout {
    pub root: Dir,
}

impl StorageLayout {

    pub fn new(root: Dir) -> Self {
        Self { root }
    }

    pub fn default_root_dir() -> Dir {
        Dir::new(PathBuf::from("var").join("lib").join("miru"))
    }

    pub fn new_default() -> Self {
        Self::new(Self::default_root_dir())
    }

    pub fn agent_cfg(&self) -> File {
        self.root.file("agent_config.json")
    }

    pub fn cfg_sch_digest_registry(&self) -> ConfigSchemaDigestRegistry {
        ConfigSchemaDigestRegistry::new(self.root.subdir("config_schema_digests"))
    }

    pub fn latest_cncr_cfg_registry(&self) -> LatestConcreteConfigRegistry {
        LatestConcreteConfigRegistry::new(self.root.subdir("concrete_configs"))
    }
}
