// internal crates
use crate::filesys::dir::Dir;
use crate::filesys::file::File;

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
        Dir::new(PathBuf::from("/").join("var").join("lib").join("miru"))
    }

    pub fn new_default() -> Self {
        Self::new(Self::default_root_dir())
    }

    pub fn agent_cfg(&self) -> File {
        self.root.file("agent_config.json")
    }

    pub fn config_schema_digest_cache(&self) -> Dir {
        self.root.subdir("config_schema_digests")
    }

    pub fn concrete_config_cache(&self) -> Dir {
        self.root.subdir("concrete_configs")
    }
}
