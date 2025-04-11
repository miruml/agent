// standard library
use std::collections::HashMap;
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File};
use crate::models::job::Job;
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================= JOB FILE ===================================== //

impl Manifest for Job {
    fn id(&self) -> &String {
        &self.id
    }

    fn id_prefix() -> &'static str {
        "job_"
    }
}

#[derive(Clone, Debug)]
pub struct JobFile {
    file: File,
    // cache the job information in memory to avoid reading the file each time
    cache: Arc<Job>,
    // store whether the job file has updates which have not been sent to the
    // server
    synced: bool,
}

impl CachedFilePrivate<Job> for JobFile {
    fn set_cache(&mut self, cache: Job) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<Job> for JobFile {
    fn init_struct(job_file: File, cache: Job) -> Self {
        Self {
            file: job_file,
            cache: Arc::new(cache),
            synced: false, // assume the job file is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "job.json"
    }

    fn cache(&self) -> Arc<Job> {
        self.cache.clone()
    }
}

impl Sync for JobFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<Job> for JobFile {}

impl ManifestFile<Job> for JobFile {}

// ================================ JOB ASSETS ================================ //
#[derive(Clone, Debug)]
pub struct JobAssets {}

impl Assets for JobAssets {
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

// ================================= JOB DIR ===================================== //
#[derive(Clone, Debug)]
pub struct JobDir {
    pub dir: Dir,
    pub job_id: String,
    pub assets: Arc<JobAssets>,
    pub job_file: JobFile,
}

impl LibraryDir<Job, JobFile, JobAssets> for JobDir {
    fn init_struct(dir: Dir, mnf_id: String, assets: JobAssets, manifest_file: JobFile) -> Self {
        Self {
            dir,
            job_id: mnf_id,
            assets: Arc::new(assets),
            job_file: manifest_file,
        }
    }

    /// Returns the name of the job directory
    fn name(manifest_id: &str) -> String {
        manifest_id.to_string()
    }

    fn get_assets(&self) -> Arc<JobAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: JobAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &JobFile {
        &self.job_file
    }

    fn read_mnf(&self) -> Arc<Job> {
        self.job_file.cache()
    }

    fn update_mnf(&mut self, manifest: Job) -> Result<(), StorageErr> {
        self.job_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.job_id
    }
}

impl Sync for JobDir {
    fn is_synced(&self) -> bool {
        self.job_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.job_file.set_synced(synced);
    }
}

// ================================= JOB LIBRARY ================================== //
#[derive(Clone, Debug)]
pub struct JobLibrary {
    pub dir: Dir,
    // contents of the job library
    job_dirs: HashMap<String, JobDir>,
}

impl LibraryPrivate<JobDir, Job, JobFile, JobAssets> for JobLibrary {
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, JobDir> {
        &mut self.job_dirs
    }
}

impl Library<JobDir, Job, JobFile, JobAssets> for JobLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, JobDir>) -> Self {
        Self {
            dir,
            job_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "jobs"
    }

    fn get_all_dirs(&self) -> &HashMap<String, JobDir> {
        &self.job_dirs
    }
}
