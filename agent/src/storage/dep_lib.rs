// standard library
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File};
use crate::models::deployment::{Deployment, DeploymentAction};
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================ DEPLOYMENT FILE ================================ //

impl Manifest for Deployment {
    fn id(&self) -> &String {
        &self.id
    }

    fn id_prefix() -> &'static str {
        "dep_"
    }
}

#[derive(Clone, Debug)]
pub struct DeploymentFile {
    file: File,
    // use to cache the deployment information in memory to avoid reading the file
    cache: Arc<Deployment>,
    // store whether the deployment file is synced with the server
    synced: bool,
}

impl CachedFilePrivate<Deployment> for DeploymentFile {
    fn set_cache(&mut self, cache: Deployment) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<Deployment> for DeploymentFile {
    fn init_struct(dep_file: File, cache: Deployment) -> Self {
        Self {
            file: dep_file,
            cache: Arc::new(cache),
            synced: false, // assume device is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    // Return the name of the deployment file
    fn file_name() -> &'static str {
        "deployment.json"
    }

    /// Read the deployment file. If the deployment file is missing fields, the default
    /// values are used.
    fn cache(&self) -> Arc<Deployment> {
        self.cache.clone()
    }
}

impl Sync for DeploymentFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<Deployment> for DeploymentFile {}

impl ManifestFile<Deployment> for DeploymentFile {}

// ================================ DEPLOYMENT ASSETS ============================== //
#[derive(Clone, Debug)]
pub struct DeploymentAssets {}

impl Assets for DeploymentAssets {
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

// ============================= DEPLOYMENT DIR ==================================== //
#[derive(Clone, Debug)]
pub struct DeploymentDir {
    pub dir: Dir,
    pub dep_id: String,
    pub assets: Arc<DeploymentAssets>,
    pub dep_file: DeploymentFile,
}

impl LibraryDir<Deployment, DeploymentFile, DeploymentAssets> for DeploymentDir {
    fn init_struct(
        dir: Dir,
        mnf_id: String,
        assets: DeploymentAssets,
        manifest_file: DeploymentFile,
    ) -> Self {
        Self {
            dir,
            dep_id: mnf_id,
            assets: Arc::new(assets),
            dep_file: manifest_file,
        }
    }

    fn get_assets(&self) -> Arc<DeploymentAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: DeploymentAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &DeploymentFile {
        &self.dep_file
    }

    fn read_mnf(&self) -> Arc<Deployment> {
        self.dep_file.cache()
    }

    fn update_mnf(&mut self, manifest: Deployment) -> Result<(), StorageErr> {
        self.dep_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.dep_id
    }
}

impl Sync for DeploymentDir {
    fn is_synced(&self) -> bool {
        self.dep_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.dep_file.set_synced(synced);
    }
}

// =============================== DEPLOYMENT LIBRARY ============================== //
#[derive(Clone, Debug)]
pub struct DeploymentLibrary {
    pub dir: Dir,
    // contents of the deployment library
    dep_dirs: HashMap<String, DeploymentDir>,
}

impl LibraryPrivate<DeploymentDir, Deployment, DeploymentFile, DeploymentAssets>
    for DeploymentLibrary
{
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, DeploymentDir> {
        &mut self.dep_dirs
    }
}

impl Library<DeploymentDir, Deployment, DeploymentFile, DeploymentAssets> for DeploymentLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, DeploymentDir>) -> Self {
        Self {
            dir,
            dep_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "deployments"
    }

    fn get_all_dirs(&self) -> &HashMap<String, DeploymentDir> {
        &self.dep_dirs
    }
}

impl DeploymentLibrary {
    /// Finds all deployments associated with the given artifact id
    pub fn find_deps_by_artifact_id(&self, artifact_id: &str) -> Vec<Arc<Deployment>> {
        let mut matching_deps: Vec<Arc<Deployment>> = Vec::new();
        for dep in self.read_all_mnfs() {
            if dep.artifact_id == artifact_id {
                matching_deps.push(dep);
            }
        }
        matching_deps
    }

    /// Retrieves all the deployments which are in the 'ongoing' state. This essentially
    /// any deployment which is not marked for removal and ready to be removed. So these
    /// deployments may be queued, downloading, stopping, etc.
    pub fn get_ongoing_deps(&self) -> Vec<Arc<Deployment>> {
        let mut matching_deps: Vec<Arc<Deployment>> = Vec::new();
        for dep in self.read_all_mnfs() {
            if dep.next_action(true) != DeploymentAction::Remove {
                matching_deps.push(dep);
            }
        }
        matching_deps
    }

    /// Retrieves all the deployments which are marked for removal and ready to be
    /// removed
    pub fn get_removable_deps(&self) -> Vec<Arc<Deployment>> {
        let mut matching_deps: Vec<Arc<Deployment>> = Vec::new();
        for dep in self.read_synced_mnfs() {
            if dep.next_action(true) == DeploymentAction::Remove {
                matching_deps.push(dep);
            }
        }
        matching_deps
    }

    /// Retrieves all the artifact ids which are from ongoing deployments
    pub fn get_ongoing_artifact_ids(&self) -> HashSet<String> {
        let mut ongoing_artifact_ids: HashSet<String> = HashSet::new();
        for dep in self.get_ongoing_deps() {
            ongoing_artifact_ids.insert(dep.artifact_id.clone());
        }
        ongoing_artifact_ids
    }
}
