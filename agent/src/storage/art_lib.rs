// standard library
use crate::filesys::path::PathExt;
use std::collections::HashMap;
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File};
use crate::models::{artifact::Artifact, image::Image};
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
use crate::trace;
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================== ARTIFACT FILE ================================= //
impl Manifest for Artifact {
    fn id(&self) -> &String {
        &self.id
    }

    fn id_prefix() -> &'static str {
        "art_"
    }
}

#[derive(Clone, Debug)]
pub struct ArtifactFile {
    file: File,
    // cache the artifact in memory to avoid reading the file each time
    cache: Arc<Artifact>,
    // store whether the artifact file is synced with the server
    synced: bool,
}

impl CachedFilePrivate<Artifact> for ArtifactFile {
    fn set_cache(&mut self, cache: Artifact) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<Artifact> for ArtifactFile {
    fn init_struct(artifact_file: File, cache: Artifact) -> Self {
        Self {
            file: artifact_file,
            cache: Arc::new(cache),
            synced: false, // assume artifact is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "artifact.json"
    }

    fn cache(&self) -> Arc<Artifact> {
        self.cache.clone()
    }
}

impl Sync for ArtifactFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<Artifact> for ArtifactFile {}

impl ManifestFile<Artifact> for ArtifactFile {}

// ================================ ARTIFACT ASSETS ================================ //
#[derive(Clone, Debug)]
pub struct ArtifactAssets {
    pub compose_file: File,
    pub env_file: Option<File>,
}

impl Assets for ArtifactAssets {
    fn new(dir: &Dir) -> Result<Self, StorageErr> {
        let env_file = dir.new_file(".env");
        let env_file = if env_file.exists() {
            Some(env_file)
        } else {
            None
        };

        let assets = Self {
            compose_file: dir.new_file("compose.yml"),
            env_file,
        };
        assets.validate()?;
        Ok(assets)
    }

    fn move_to(&mut self, dir: &Dir, overwrite: bool) -> Result<(), StorageErr> {
        self.validate()?;

        let new_compose_file = dir.new_file("compose.yml");
        self.compose_file
            .move_to(&new_compose_file, overwrite)
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;

        if let Some(env_file) = &self.env_file {
            let new_env_file = dir.new_file(".env");
            env_file
                .move_to(&new_env_file, overwrite)
                .map_err(|e| StorageErr::FileSysErr {
                    source: e,
                    trace: trace!(),
                })?;
            self.env_file = Some(new_env_file);
        }

        self.compose_file = new_compose_file;
        self.validate()?;
        Ok(())
    }

    /// Validate all artifact assets by ensuring their contents are valid.
    fn validate(&self) -> Result<(), StorageErr> {
        self.validate_compose_file()?;
        self.validate_env_file()?;
        Ok(())
    }
}

impl ArtifactAssets {
    /// Validate the compose file by ensuring it exists. YAML libraries in Rust are not
    /// stable, so validating the compose file's YAML format creates more problems than
    /// it solves. For now, the compose file won't be known to be truly valid until it
    /// is parsed by the Docker client.
    pub fn validate_compose_file(&self) -> Result<(), StorageErr> {
        // Ensure the compose file exists
        let _: Vec<u8> = self
            .compose_file
            .read_bytes()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(())
    }

    pub fn validate_env_file(&self) -> Result<(), StorageErr> {
        if let Some(env_file) = &self.env_file {
            let _: Vec<u8> = env_file.read_bytes().map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        }
        Ok(())
    }
}

// ================================== ARTIFACT DIR ================================= //
#[derive(Clone, Debug)]
pub struct ArtifactDir {
    pub dir: Dir,
    pub artifact_id: String,
    pub assets: Arc<ArtifactAssets>,
    pub artifact_file: ArtifactFile,
}

impl Sync for ArtifactDir {
    fn is_synced(&self) -> bool {
        self.artifact_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.artifact_file.set_synced(synced);
    }
}

impl LibraryDir<Artifact, ArtifactFile, ArtifactAssets> for ArtifactDir {
    fn init_struct(
        dir: Dir,
        mnf_id: String,
        assets: ArtifactAssets,
        manifest_file: ArtifactFile,
    ) -> Self {
        Self {
            dir,
            artifact_id: mnf_id,
            assets: Arc::new(assets),
            artifact_file: manifest_file,
        }
    }

    /// Returns the name of the artifact directory
    fn name(art_id: &str) -> String {
        art_id.to_string()
    }

    fn get_assets(&self) -> Arc<ArtifactAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: ArtifactAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &ArtifactFile {
        &self.artifact_file
    }

    fn read_mnf(&self) -> Arc<Artifact> {
        self.artifact_file.cache()
    }

    fn update_mnf(&mut self, manifest: Artifact) -> Result<(), StorageErr> {
        self.artifact_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.artifact_id
    }
}

// ================================ ARTIFACT LIBRARY =============================== //
#[derive(Clone, Debug)]
pub struct ArtifactLibrary {
    pub dir: Dir,
    // contents of the artifact library
    artifact_dirs: HashMap<String, ArtifactDir>,
}

impl ArtifactLibrary {
    /// Returns a map from the image id to the image of every image of every artifact in
    /// the library
    pub fn get_images_map(&self) -> HashMap<String, Image> {
        let artifacts = self.read_all_mnfs();

        let mut images: HashMap<String, Image> = HashMap::new();
        for artifact in artifacts.into_iter() {
            for image in artifact.images.iter() {
                images.insert(image.id.clone(), image.clone());
            }
        }
        images
    }
}

impl LibraryPrivate<ArtifactDir, Artifact, ArtifactFile, ArtifactAssets> for ArtifactLibrary {
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, ArtifactDir> {
        &mut self.artifact_dirs
    }
}

impl Library<ArtifactDir, Artifact, ArtifactFile, ArtifactAssets> for ArtifactLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, ArtifactDir>) -> Self {
        Self {
            dir,
            artifact_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "artifacts"
    }

    fn get_all_dirs(&self) -> &HashMap<String, ArtifactDir> {
        &self.artifact_dirs
    }
}

impl ArtifactLibrary {
    /// Finds all artifacts created from the given source
    pub fn find_artifacts_by_source(&self, source_id: &str) -> Vec<Arc<Artifact>> {
        let mut matching_artifacts: Vec<Arc<Artifact>> = Vec::new();
        let all_artifacts = self.read_all_mnfs();
        for artifact in all_artifacts {
            if artifact.source_id == source_id {
                matching_artifacts.push(artifact);
            }
        }
        matching_artifacts
    }

    /// Returns a list of all the images in the library with the given artifact ids
    pub fn get_images<I>(&self, artifact_ids: I) -> Result<Vec<Image>, StorageErr>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut images: Vec<Image> = Vec::new();
        for artifact in self.read_mnfs(artifact_ids)? {
            for image in artifact.images.iter() {
                images.push(image.clone());
            }
        }
        Ok(images)
    }

    pub fn get_compose_file(&self, artifact_id: &str) -> Result<File, StorageErr> {
        let assets = self.get_assets(artifact_id)?;
        assets
            .compose_file
            .assert_exists()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(assets.compose_file.clone())
    }
}
