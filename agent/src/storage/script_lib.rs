// standard library
use std::collections::HashMap;
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, file::File};
use crate::models::job::Job;
use crate::models::script::Script;
use crate::storage::{
    cached_file::CachedFilePrivate, errors::StorageErr, library::LibraryPrivate, prelude::*,
};
use crate::trace;
// external crates
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================= SCRIPT FILE ================================= //
impl Manifest for Script {
    fn id(&self) -> &String {
        &self.id
    }

    fn id_prefix() -> &'static str {
        "script_"
    }
}

#[derive(Clone, Debug)]
pub struct ScriptFile {
    file: File,
    // cache the script information in memory to avoid reading the file each time
    cache: Arc<Script>,
    // store whether the script file has updates which have not been sent to the
    // server
    synced: bool,
}

impl CachedFilePrivate<Script> for ScriptFile {
    fn set_cache(&mut self, cache: Script) {
        self.cache = Arc::new(cache);
    }
}

impl CachedFile<Script> for ScriptFile {
    fn init_struct(script_file: File, cache: Script) -> Self {
        Self {
            file: script_file,
            cache: Arc::new(cache),
            synced: false, // assume the script file is not synced initially
        }
    }

    fn file(&self) -> &File {
        &self.file
    }

    fn file_permissions() -> u32 {
        0o600
    }

    fn file_name() -> &'static str {
        "script.json"
    }

    fn cache(&self) -> Arc<Script> {
        self.cache.clone()
    }
}

impl Sync for ScriptFile {
    fn is_synced(&self) -> bool {
        self.synced
    }

    fn set_synced(&mut self, synced: bool) {
        self.synced = synced;
    }
}

impl SyncedFile<Script> for ScriptFile {}

impl ManifestFile<Script> for ScriptFile {}

// ================================ SCRIPT ASSETS ================================ //
#[derive(Clone, Debug)]
pub struct ScriptAssets {
    pub script_file: File,
}

impl Assets for ScriptAssets {
    fn new(dir: &Dir) -> Result<Self, StorageErr> {
        let assets = Self {
            script_file: dir.new_file("script.sh"),
        };
        assets.validate()?;
        Ok(assets)
    }

    fn move_to(&mut self, dir: &Dir, overwrite: bool) -> Result<(), StorageErr> {
        self.validate()?;

        let new_script_file = dir.new_file("script.sh");
        self.script_file
            .move_to(&new_script_file, overwrite)
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;

        self.script_file = new_script_file;
        self.validate()?;
        Ok(())
    }

    fn validate(&self) -> Result<(), StorageErr> {
        self.validate_script_file()?;
        Ok(())
    }
}

impl ScriptAssets {
    pub fn validate_script_file(&self) -> Result<(), StorageErr> {
        let _: Vec<u8> = self
            .script_file
            .read_bytes()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(())
    }
}

// ================================= SCRIPT DIR =================================== //
#[derive(Clone, Debug)]
pub struct ScriptDir {
    pub dir: Dir,
    pub script_id: String,
    pub assets: Arc<ScriptAssets>,
    pub script_file: ScriptFile,
}

impl LibraryDir<Script, ScriptFile, ScriptAssets> for ScriptDir {
    fn init_struct(
        dir: Dir,
        mnf_id: String,
        assets: ScriptAssets,
        manifest_file: ScriptFile,
    ) -> Self {
        Self {
            dir,
            script_id: mnf_id,
            assets: Arc::new(assets),
            script_file: manifest_file,
        }
    }

    /// Returns the name of the script directory
    fn name(manifest_id: &str) -> String {
        manifest_id.to_string()
    }

    fn get_assets(&self) -> Arc<ScriptAssets> {
        self.assets.clone()
    }

    fn set_assets(&mut self, assets: ScriptAssets) -> Result<(), StorageErr> {
        self.assets = Arc::new(assets);
        Ok(())
    }

    fn get_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_mnf_file(&self) -> &ScriptFile {
        &self.script_file
    }

    fn read_mnf(&self) -> Arc<Script> {
        self.script_file.cache()
    }

    fn update_mnf(&mut self, manifest: Script) -> Result<(), StorageErr> {
        self.script_file.synced_write(manifest)?;
        Ok(())
    }

    fn id(&self) -> &str {
        &self.script_id
    }
}

impl Sync for ScriptDir {
    fn is_synced(&self) -> bool {
        self.script_file.is_synced()
    }

    fn set_synced(&mut self, synced: bool) {
        self.script_file.set_synced(synced);
    }
}

// ================================= SCRIPT LIBRARY ============================== //
#[derive(Clone, Debug)]
pub struct ScriptLibrary {
    pub dir: Dir,
    // contents of the script library
    script_dirs: HashMap<String, ScriptDir>,
}

impl LibraryPrivate<ScriptDir, Script, ScriptFile, ScriptAssets> for ScriptLibrary {
    fn get_lib_dir(&self) -> &Dir {
        &self.dir
    }

    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, ScriptDir> {
        &mut self.script_dirs
    }
}

impl Library<ScriptDir, Script, ScriptFile, ScriptAssets> for ScriptLibrary {
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, ScriptDir>) -> Self {
        Self {
            dir,
            script_dirs: lib_dirs,
        }
    }

    fn name() -> &'static str {
        "scripts"
    }

    fn get_all_dirs(&self) -> &HashMap<String, ScriptDir> {
        &self.script_dirs
    }
}

impl ScriptLibrary {
    pub fn read_mnfs_for_job(&self, job: &Job) -> Result<Vec<Arc<Script>>, StorageErr> {
        let script_ids = job.script_ids();
        self.read_mnfs(script_ids.iter())
    }
}
