// standard library
use std::collections::HashMap;
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File};
use crate::models::job_run::JobRun;
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================= JOB RUN FILE ================================= //

impl Manifest for JobRun {
    fn id(&self) -> &String {
        &self.id
    }

    fn id_prefix() -> &'static str {
        "job_run_"
    }
}

#[derive(Clone, Debug)]
pub struct JobRunFile {
    file: File,
    // cache the job run information in memory to avoid reading the file each time
    cache: Arc<JobRun>,
    // store whether the job run file has updates which have not been sent to the
    // server
    synced: bool,
}

impl CachedFilePrivate<JobRun> for JobRunFile {
    fn set_cache(&mut self, cache: JobRun) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<JobRun> for JobRunFile {
    fn init_struct(job_run_file: File, cache: JobRun) -> Self {
        Self {
            file: job_run_file,
            cache: Arc::new(cache),
            synced: false, // assume the job run file is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "job_run.json"
    }

    fn cache(&self) -> Arc<JobRun> {
        self.cache.clone()
    }
}

impl Sync for JobRunFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<JobRun> for JobRunFile {}

impl ManifestFile<JobRun> for JobRunFile {}

// ================================ JOB RUN ASSETS ================================ //
#[derive(Clone, Debug)]
pub struct JobRunAssets {}

impl Assets for JobRunAssets {
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

// ================================= JOB RUN DIR =================================== //
#[derive(Clone, Debug)]
pub struct JobRunDir {
    pub dir: Dir,
    pub job_run_id: String,
    pub assets: Arc<JobRunAssets>,
    pub job_run_file: JobRunFile,
}

impl LibraryDir<JobRun, JobRunFile, JobRunAssets> for JobRunDir {
    fn init_struct(
        dir: Dir,
        mnf_id: String,
        assets: JobRunAssets,
        manifest_file: JobRunFile,
    ) -> Self {
        Self {
            dir,
            job_run_id: mnf_id,
            assets: Arc::new(assets),
            job_run_file: manifest_file,
        }
    }

    /// Returns the name of the job run directory
    fn name(manifest_id: &str) -> String {
        manifest_id.to_string()
    }

    fn get_assets(&self) -> Arc<JobRunAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: JobRunAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &JobRunFile {
        &self.job_run_file
    }

    fn read_mnf(&self) -> Arc<JobRun> {
        self.job_run_file.cache()
    }

    fn update_mnf(&mut self, manifest: JobRun) -> Result<(), StorageErr> {
        self.job_run_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.job_run_id
    }
}

impl Sync for JobRunDir {
    fn is_synced(&self) -> bool {
        self.job_run_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.job_run_file.set_synced(synced);
    }
}

// ================================= JOB RUN LIBRARY ============================== //
#[derive(Clone, Debug)]
pub struct JobRunLibrary {
    pub dir: Dir,
    // contents of the job run library
    job_run_dirs: HashMap<String, JobRunDir>,
}

impl LibraryPrivate<JobRunDir, JobRun, JobRunFile, JobRunAssets> for JobRunLibrary {
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, JobRunDir> {
        &mut self.job_run_dirs
    }
}

impl Library<JobRunDir, JobRun, JobRunFile, JobRunAssets> for JobRunLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, JobRunDir>) -> Self {
        Self {
            dir,
            job_run_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "job_runs"
    }

    fn get_all_dirs(&self) -> &HashMap<String, JobRunDir> {
        &self.job_run_dirs
    }
}
