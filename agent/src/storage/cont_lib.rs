use std::collections::HashMap;
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File};
use crate::models::container::Container;
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// =============================== CONTAINER FILE ================================== //

impl Manifest for Container {
    fn id(&self) -> &String {
        &self.docker_id
    }

    fn id_prefix() -> &'static str {
        "cont_"
    }
}

#[derive(Clone, Debug)]
pub struct ContainerFile {
    file: File,
    // cache the containers
    cache: Arc<Container>,
    // store whether the containers file has updates which have not been sent to the
    // server
    synced: bool,
}

impl CachedFilePrivate<Container> for ContainerFile {
    fn set_cache(&mut self, cache: Container) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<Container> for ContainerFile {
    fn init_struct(containers_file: File, cache: Container) -> Self {
        Self {
            file: containers_file,
            cache: Arc::new(cache),
            synced: false, // assume the containers file is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "container.json"
    }

    fn cache(&self) -> Arc<Container> {
        self.cache.clone()
    }
}

impl Sync for ContainerFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<Container> for ContainerFile {}

impl ManifestFile<Container> for ContainerFile {}

// ================================ CONTAINER ASSETS =============================== //
#[derive(Clone, Debug)]
pub struct ContainerAssets {}

impl Assets for ContainerAssets {
    fn new(_: &Dir) -> Result<Self, StorageErr> {
        Ok(Self {})
    }

    fn move_to(&mut self, _: &Dir, _: bool) -> Result<(), StorageErr> {
        Ok(())
    }

    fn validate(&self) -> Result<(), StorageErr> {
        Ok(())
    }
}

// ================================ CONTAINER DIR ================================== //
#[derive(Clone, Debug)]
pub struct ContainerDir {
    pub dir: Dir,
    pub container_id: String,
    pub assets: Arc<ContainerAssets>,
    pub container_file: ContainerFile,
}

impl LibraryDir<Container, ContainerFile, ContainerAssets> for ContainerDir {
    fn init_struct(
        dir: Dir,
        mnf_id: String,
        assets: ContainerAssets,
        manifest_file: ContainerFile,
    ) -> Self {
        Self {
            dir,
            container_id: mnf_id,
            assets: Arc::new(assets),
            container_file: manifest_file,
        }
    }

    fn get_assets(&self) -> Arc<ContainerAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: ContainerAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &ContainerFile {
        &self.container_file
    }

    fn read_mnf(&self) -> Arc<Container> {
        self.container_file.cache()
    }

    fn update_mnf(&mut self, manifest: Container) -> Result<(), StorageErr> {
        self.container_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.container_id
    }
}

impl Sync for ContainerDir {
    fn is_synced(&self) -> bool {
        self.container_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.container_file.set_synced(synced);
    }
}

// ================================ CONTAINER LIBRARY =============================== //
#[derive(Clone, Debug)]
pub struct ContainerLibrary {
    pub dir: Dir,
    // contents of the container library
    container_dirs: HashMap<String, ContainerDir>,
}

impl LibraryPrivate<ContainerDir, Container, ContainerFile, ContainerAssets> for ContainerLibrary {
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, ContainerDir> {
        &mut self.container_dirs
    }
}

impl Library<ContainerDir, Container, ContainerFile, ContainerAssets> for ContainerLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, ContainerDir>) -> Self {
        Self {
            dir,
            container_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "containers"
    }

    fn get_all_dirs(&self) -> &HashMap<String, ContainerDir> {
        &self.container_dirs
    }
}

impl ContainerLibrary {
    pub fn get_removable_conts(&self) -> Vec<Arc<Container>> {
        let mut conts: Vec<Arc<Container>> = Vec::new();
        for cont in self.read_synced_mnfs() {
            if cont.deleted {
                conts.push(cont);
            }
        }
        conts
    }
}
