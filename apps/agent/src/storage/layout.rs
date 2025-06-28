// standard library
use std::path::PathBuf;

// internal crates
use crate::filesys::dir::Dir;
use crate::filesys::file::File;

// external crates
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

    pub fn auth_dir(&self) -> AuthLayout {
        AuthLayout::new(self.root.subdir("auth"))
    }

    pub fn agent_file(&self) -> File {
        self.root.file("agent.json")
    }

    pub fn caches_dir(&self) -> Dir {
        self.root.subdir("cache")
    }

    pub fn config_schema_caches(&self) -> Dir {
        self.caches_dir().subdir("config_schemas")
    }

    pub fn config_schema_digest_cache(&self) -> File {
        self.config_schema_caches().file("digests.json")
    }

    pub fn config_schema_cache(&self) -> File {
        self.config_schema_caches().file("metadata.json")
    }

    pub fn config_instance_caches(&self) -> Dir {
        self.caches_dir().subdir("config_instances")
    }

    pub fn config_instance_metadata_cache(&self) -> File {
        self.config_instance_caches().file("metadata.json")
    }

    pub fn config_instance_data_cache(&self) -> Dir {
        self.config_instance_caches().subdir("instances")
    }


}

impl Default for StorageLayout {
    fn default() -> Self {
        Self::new(Self::default_root_dir())
    }
}

pub struct AuthLayout {
    pub root: Dir,
}

impl AuthLayout {
    pub fn new(root: Dir) -> Self {
        Self { root }
    }

    pub fn private_key_file(&self) -> File {
        self.root.file("private_key.pem")
    }

    pub fn public_key_file(&self) -> File {
        self.root.file("public_key.pem")
    }

    pub fn token_file(&self) -> File {
        self.root.file("token.json")
    }
}
