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

    pub fn config_schema_digest_cache(&self) -> Dir {
        self.root.subdir("config_schema_digests")
    }

    pub fn config_instance_cache(&self) -> Dir {
        self.root.subdir("config_instances")
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
