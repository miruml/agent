// standard library
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File, path::PathExt};
use crate::models::{job_run::JobRun, script_run::ScriptRun};
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
use crate::trace;
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================ SCRIPT RUN FILE ================================ //
impl Manifest for ScriptRun {
    fn id(&self) -> &String {
        &self.id
    }

    fn id_prefix() -> &'static str {
        "script_run_"
    }
}

#[derive(Clone, Debug)]
pub struct ScriptRunFile {
    file: File,
    // cache the script run information in memory to avoid reading the file each time
    cache: Arc<ScriptRun>,
    // store whether the script run file has updates which have not been sent to the
    // server
    synced: bool,
}

impl CachedFilePrivate<ScriptRun> for ScriptRunFile {
    fn set_cache(&mut self, cache: ScriptRun) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<ScriptRun> for ScriptRunFile {
    fn init_struct(script_run_file: File, cache: ScriptRun) -> Self {
        Self {
            file: script_run_file,
            cache: Arc::new(cache),
            synced: false, // assume the script run file is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "script_run.json"
    }

    fn cache(&self) -> Arc<ScriptRun> {
        self.cache.clone()
    }
}

impl Sync for ScriptRunFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<ScriptRun> for ScriptRunFile {}

impl ManifestFile<ScriptRun> for ScriptRunFile {}

// ================================ SCRIPT RUN ASSETS ============================= //
#[derive(Clone, Debug)]
pub struct ScriptRunAssets {
    pub stdout: Option<File>,
    pub stderr: Option<File>,
}

impl Assets for ScriptRunAssets {
    fn new(dir: &Dir) -> Result<Self, StorageErr> {
        let stdout = dir.new_file(Self::stdout_filename());
        let stdout = if stdout.exists() { Some(stdout) } else { None };

        let stderr = dir.new_file("stderr.txt");
        let stderr = if stderr.exists() { Some(stderr) } else { None };

        let assets = Self { stdout, stderr };
        assets.validate()?;
        Ok(assets)
    }

    fn move_to(&mut self, dir: &Dir, overwrite: bool) -> Result<(), StorageErr> {
        self.validate()?;

        // move stdout
        if let Some(stdout) = &self.stdout {
            let new_stdout = dir.new_file(Self::stdout_filename());
            stdout
                .move_to(&new_stdout, overwrite)
                .map_err(|e| StorageErr::FileSysErr {
                    source: e,
                    trace: trace!(),
                })?;
            self.stdout = Some(new_stdout);
        };

        // move stderr
        if let Some(stderr) = &self.stderr {
            let new_stderr = dir.new_file(Self::stderr_filename());
            stderr
                .move_to(&new_stderr, overwrite)
                .map_err(|e| StorageErr::FileSysErr {
                    source: e,
                    trace: trace!(),
                })?;
            self.stderr = Some(new_stderr);
        };

        self.validate()?;
        Ok(())
    }

    fn validate(&self) -> Result<(), StorageErr> {
        self.validate_stdout()?;
        self.validate_stderr()?;
        Ok(())
    }
}

impl ScriptRunAssets {
    pub fn stdout_filename() -> &'static str {
        "stdout.txt"
    }

    pub fn stderr_filename() -> &'static str {
        "stderr.txt"
    }

    pub fn validate_stdout(&self) -> Result<(), StorageErr> {
        if let Some(stdout) = &self.stdout {
            let _: Vec<u8> = stdout.read_bytes().map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        }
        Ok(())
    }

    pub fn validate_stderr(&self) -> Result<(), StorageErr> {
        if let Some(stderr) = &self.stderr {
            let _: Vec<u8> = stderr.read_bytes().map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        }
        Ok(())
    }
}

// ================================= SCRIPT RUN DIR ================================ //
#[derive(Clone, Debug)]
pub struct ScriptRunDir {
    pub dir: Dir,
    pub script_run_id: String,
    pub assets: Arc<ScriptRunAssets>,
    pub script_run_file: ScriptRunFile,
}

impl LibraryDir<ScriptRun, ScriptRunFile, ScriptRunAssets> for ScriptRunDir {
    fn init_struct(
        dir: Dir,
        mnf_id: String,
        assets: ScriptRunAssets,
        manifest_file: ScriptRunFile,
    ) -> Self {
        Self {
            dir,
            script_run_id: mnf_id,
            assets: Arc::new(assets),
            script_run_file: manifest_file,
        }
    }

    fn get_assets(&self) -> Arc<ScriptRunAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: ScriptRunAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &ScriptRunFile {
        &self.script_run_file
    }

    fn read_mnf(&self) -> Arc<ScriptRun> {
        self.script_run_file.cache()
    }

    fn update_mnf(&mut self, manifest: ScriptRun) -> Result<(), StorageErr> {
        self.script_run_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.script_run_id
    }
}

impl Sync for ScriptRunDir {
    fn is_synced(&self) -> bool {
        self.script_run_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.script_run_file.set_synced(synced);
    }
}

// ================================= SCRIPT RUN LIBRARY =========================== //
#[derive(Clone, Debug)]
pub struct ScriptRunLibrary {
    pub dir: Dir,
    // contents of the script run library
    script_run_dirs: HashMap<String, ScriptRunDir>,
}

impl LibraryPrivate<ScriptRunDir, ScriptRun, ScriptRunFile, ScriptRunAssets> for ScriptRunLibrary {
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, ScriptRunDir> {
        &mut self.script_run_dirs
    }
}

impl Library<ScriptRunDir, ScriptRun, ScriptRunFile, ScriptRunAssets> for ScriptRunLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, ScriptRunDir>) -> Self {
        Self {
            dir,
            script_run_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "script_runs"
    }

    fn get_all_dirs(&self) -> &HashMap<String, ScriptRunDir> {
        &self.script_run_dirs
    }
}

impl ScriptRunLibrary {
    pub fn read_mnfs_for_job_run(
        &self,
        job_run: &JobRun,
    ) -> Result<Vec<Arc<ScriptRun>>, StorageErr> {
        let script_run_ids = job_run.script_run_ids();
        self.read_mnfs(script_run_ids.iter())
    }

    pub fn read_mnfs_not_from_job_runs(
        &self,
        job_runs: &HashSet<String>,
    ) -> Result<Vec<Arc<ScriptRun>>, StorageErr> {
        let mut script_runs: Vec<Arc<ScriptRun>> = Vec::new();
        for script_run in self.read_all_mnfs() {
            if !job_runs.contains(&script_run.job_run_id) {
                script_runs.push(script_run);
            }
        }
        Ok(script_runs)
    }
}
