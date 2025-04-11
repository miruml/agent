// standard library
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
// internal crates
use crate::filesys::{dir::Dir, path::PathExt};
use crate::storage::{errors::StorageErr, prelude::*};
use crate::trace;
// external crates
use serde::{de::DeserializeOwned, Serialize};
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// ================================= MANIFEST FILE ================================= //
pub trait Manifest
where
    Self: Sized + Clone + Default + Serialize + DeserializeOwned,
{
    fn id(&self) -> &String;
    fn id_prefix() -> &'static str;
}

pub trait ManifestFile<ManifestT>
where
    ManifestT: Manifest,
    Self: Sized + SyncedFile<ManifestT>,
{
}

// ===================================== ASSETS ==================================== //
pub trait Assets
where
    Self: Sized + Clone,
{
    fn new(dir: &Dir) -> Result<Self, StorageErr>;
    fn move_to(&mut self, dir: &Dir, overwrite: bool) -> Result<(), StorageErr>;
    fn validate(&self) -> Result<(), StorageErr>;
}

// ================================== LIBRARY DIR ================================= //
#[allow(private_bounds)]
pub trait LibraryDir<ManifestT, ManifestFileT, AssetsT>
where
    AssetsT: Assets,
    ManifestT: Manifest,
    ManifestFileT: ManifestFile<ManifestT>,
    Self: Sized + Sync,
{
    fn init_struct(dir: Dir, mnf_id: String, assets: AssetsT, manifest_file: ManifestFileT)
        -> Self;
    fn get_assets(&self) -> Arc<AssetsT>;
    fn set_assets(&mut self, assets: AssetsT) -> Result<(), StorageErr>;
    fn get_dir(&self) -> &Dir;
    fn get_mnf_file(&self) -> &ManifestFileT;
    fn read_mnf(&self) -> Arc<ManifestT>;
    fn update_mnf(&mut self, manifest: ManifestT) -> Result<(), StorageErr>;

    fn id(&self) -> &str;

    fn name(mnf_id: &str) -> String {
        mnf_id.to_string()
    }

    fn new(dir: Dir) -> Result<Self, StorageErr> {
        // obtain the manifest id
        let mnf_id = Self::validate_dir_name(&dir)?.to_string();

        // initialize the manifest file
        let file = dir.new_file(ManifestFileT::file_name());
        let manifest_file = ManifestFileT::new(file)?;

        // initialize the assets
        let assets = AssetsT::new(&dir)?;

        // initialize the library directory
        let lib_dir = Self::init_struct(dir, mnf_id, assets, manifest_file);
        lib_dir.validate()?;
        Ok(lib_dir)
    }

    /// Creates a new directory and manifest file for the given manifest and assets.
    /// If the directory already exists, it is overwritten with the new contents.
    /// This is a default implementation which assumes the directory's only asset is
    /// the manifest file.
    fn create(dir: Dir, manifest: &ManifestT, mut assets: AssetsT) -> Result<Self, StorageErr> {
        // create the directory
        dir.delete().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        dir.create_if_absent().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;

        // create the manifest file
        let file = dir.new_file(ManifestFileT::file_name());
        ManifestFileT::create(file, manifest)?;

        // move the assets to the directory
        assets.move_to(&dir, true)?;

        Self::new(dir)
    }

    fn update_assets(&mut self, mut assets: AssetsT, overwrite: bool) -> Result<(), StorageErr> {
        let dir = self.get_dir();
        assets.move_to(dir, overwrite)?;
        self.set_assets(assets)?;
        Ok(())
    }

    /// Checks if the library directorie's name is valid. Returns the name of the
    /// directory if it is valid.
    fn validate_name(name: &str) -> Result<&str, StorageErr> {
        Dir::assert_valid_dir_name(name).map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(name)
    }

    /// Checks if the library directory's name is valid. Returns the name of the
    /// directory if it is valid.
    fn validate_dir_name(dir: &Dir) -> Result<&str, StorageErr> {
        let name = dir.name().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Self::validate_name(name)
    }

    fn validate(&self) -> Result<(), StorageErr> {
        // validate the library directory itself
        self.get_dir()
            .assert_exists()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Self::validate_dir_name(self.get_dir())?;

        // validate the manifest file
        self.get_mnf_file().validate()?;

        // validate the assets
        self.get_assets().validate()?;

        Ok(())
    }
}

// ==================================== LIBRARY ==================================== //
pub(super) trait LibraryPrivate<DirT, ManifestT, ManifestFileT, AssetsT>
where
    AssetsT: Assets,
    DirT: LibraryDir<ManifestT, ManifestFileT, AssetsT>,
    ManifestT: Manifest,
    ManifestFileT: ManifestFile<ManifestT>,
    Self: Sized,
{
    fn get_lib_dir(&self) -> &Dir;
    fn get_all_dirs_mut(&mut self) -> &mut HashMap<String, DirT>;
}

#[allow(private_bounds)]
pub trait Library<DirT, ManifestT, ManifestFileT, AssetsT>
where
    AssetsT: Assets,
    DirT: LibraryDir<ManifestT, ManifestFileT, AssetsT> + Debug,
    ManifestT: Manifest,
    ManifestFileT: ManifestFile<ManifestT>,
    Self: LibraryPrivate<DirT, ManifestT, ManifestFileT, AssetsT> + Sized,
{
    fn init_struct(dir: Dir, lib_dirs: HashMap<String, DirT>) -> Self;
    fn name() -> &'static str;
    fn get_all_dirs(&self) -> &HashMap<String, DirT>;

    /// Initialize the library at the given Dir instance. This directory
    /// must exist with valid contents or an error will be thrown.
    fn new(dir: Dir) -> Result<Self, StorageErr> {
        dir.assert_exists().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        let lib_dirs = Self::get_valid_lib_dirs(&dir)?;
        let lib = Self::init_struct(dir, lib_dirs);
        lib.validate()?;
        Ok(lib)
    }

    /// Create the library directory at the given Dir instance. If the directory already
    /// exists, nothing happens. If it doesn't exist, it is created.
    fn create_if_absent(dir: Dir) -> Result<Self, StorageErr> {
        dir.create_if_absent().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Self::new(dir)
    }

    /// Returns all the valid directories in the library as a map from the directory id
    /// to the directory
    fn get_valid_lib_dirs(lib_dir: &Dir) -> Result<HashMap<String, DirT>, StorageErr> {
        // get all subdirs
        let subdirs = lib_dir.list_subdirs().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;

        // return the valid directories
        let mut t_dirs = HashMap::with_capacity(subdirs.len());
        for subdir in subdirs.into_iter() {
            let result = DirT::new(subdir);
            if let Ok(dir) = result {
                t_dirs.insert(dir.id().to_string(), dir);
            }
        }

        Ok(t_dirs)
    }

    /// Validate the library's directory name is the expected name
    fn validate_name(name: &str) -> Result<(), StorageErr> {
        if name != Self::name() {
            return Err(StorageErr::InvalidDirName {
                name: name.to_string(),
                expected_name: Some(Self::name().to_string()),
                trace: trace!(),
            });
        }
        Ok(())
    }

    /// Validate the library's directory name is the expected name
    fn validate_dir_name(dir: &Dir) -> Result<(), StorageErr> {
        let name = dir.name().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Self::validate_name(name)
    }

    /// Validate the library by ensuring its contents are valid.
    fn validate(&self) -> Result<(), StorageErr> {
        let dir = self.get_lib_dir();
        dir.assert_exists().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Self::validate_dir_name(dir)?;

        // validate all the directories in the library
        for dir in self.get_all_dirs().values() {
            dir.validate()?;
        }
        Ok(())
    }

    /// Validate the directory with the given id
    fn validate_dir(&self, id: &str) -> Result<(), StorageErr> {
        let dir = self.get_dir(id)?;
        dir.validate()?;
        Ok(())
    }

    /// Create a new entry (directory) in the library. If the entry (directory) already
    /// exists, it is overwritten with the new contents.
    fn create_dir(
        &mut self,
        manifest: &ManifestT,
        assets: AssetsT,
    ) -> Result<&mut DirT, StorageErr> {
        let dir = self.get_lib_dir().new_subdir(&[&DirT::name(manifest.id())]);
        let t_dir = DirT::create(dir, manifest, assets)?;

        // update the list of directories
        let dir_map = self.get_all_dirs_mut();
        dir_map.insert(manifest.id().to_string(), t_dir);
        self.get_dir_mut(manifest.id())
    }

    /// Find and return a specific directory. Returns None if the directory
    /// doesn't exist.
    fn find_dir_mut(&mut self, id: &str) -> Option<&mut DirT> {
        self.get_all_dirs_mut().get_mut(id)
    }

    /// Find and return a specific directory. Returns None if the directory
    /// doesn't exist.
    fn find_dir(&self, id: &str) -> Option<&DirT> {
        self.get_all_dirs().get(id)
    }

    /// Find and return the directory associated with the given directory id. Returns
    /// None if the directory doesn't exist.
    fn get_dir_mut(&mut self, id: &str) -> Result<&mut DirT, StorageErr> {
        let result = self.find_dir_mut(id);
        match result {
            Some(dir) => Ok(dir),
            None => Err(StorageErr::LibraryDirNotFound {
                id: id.to_string(),
                trace: trace!(),
            }),
        }
    }

    /// Find and return the directory associated with the given directory id. Returns
    /// None if the directory doesn't exist.
    fn get_dir(&self, id: &str) -> Result<&DirT, StorageErr> {
        let result = self.find_dir(id);
        match result {
            Some(dir) => Ok(dir),
            None => Err(StorageErr::LibraryDirNotFound {
                id: id.to_string(),
                trace: trace!(),
            }),
        }
    }

    /// Returns a list of all the directories with the given ids
    fn get_dirs<I>(&self, ids: I) -> Result<Vec<&DirT>, StorageErr>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut dirs: Vec<&DirT> = Vec::new();
        for id in ids {
            let dir = self.get_dir(id.as_ref())?;
            dirs.push(dir);
        }
        Ok(dirs)
    }

    /// Read the manifest associated with the given manifest id
    fn read_mnf(&self, id: &str) -> Result<Arc<ManifestT>, StorageErr> {
        let dir = self.get_dir(id)?;
        let manifest = dir.read_mnf();
        Ok(manifest)
    }

    /// Returns a list of all the manifests with the given ids in the library. If any
    /// manifests are not found, an error is returned.
    fn read_mnfs<I>(&self, ids: I) -> Result<Vec<Arc<ManifestT>>, StorageErr>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut manifests: Vec<Arc<ManifestT>> = Vec::new();
        for id in ids {
            manifests.push(self.read_mnf(id.as_ref())?);
        }
        Ok(manifests)
    }

    /// Returns a list of all the manifests in the library
    fn read_all_mnfs(&self) -> Vec<Arc<ManifestT>> {
        let dirs = self.get_all_dirs();
        let mut manifests: Vec<Arc<ManifestT>> = Vec::with_capacity(dirs.len());
        for dir in dirs.values() {
            let manifest = dir.read_mnf();
            manifests.push(manifest);
        }
        manifests
    }

    /// Returns a map from the manifest id to the manifest of every manifest in the
    /// library
    fn read_all_mnfs_map(&self) -> HashMap<String, Arc<ManifestT>> {
        let mnfs = self.read_all_mnfs();
        let mut mnfs_map: HashMap<String, Arc<ManifestT>> = HashMap::with_capacity(mnfs.len());
        for mnf in mnfs.into_iter() {
            mnfs_map.insert(mnf.id().clone(), mnf);
        }
        mnfs_map
    }

    /// Updates the manifest in the directory with the manifest's given id
    fn update_mnf(&mut self, manifest: ManifestT) -> Result<(), StorageErr> {
        let dir = self.get_dir_mut(manifest.id())?;
        dir.update_mnf(manifest)?;
        Ok(())
    }

    /// Updates the manifests in the directory with the manifest's given ids
    fn update_mnfs<I>(&mut self, manifests: I) -> Result<(), StorageErr>
    where
        I: IntoIterator<Item = ManifestT>,
    {
        for manifest in manifests.into_iter() {
            self.update_mnf(manifest)?;
        }
        Ok(())
    }

    fn get_assets(&self, id: &str) -> Result<Arc<AssetsT>, StorageErr> {
        let dir = self.get_dir(id)?;
        let assets = dir.get_assets();
        Ok(assets)
    }

    fn update_assets(
        &mut self,
        id: &str,
        assets: AssetsT,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let dir = self.get_dir_mut(id)?;
        dir.update_assets(assets, overwrite)?;
        Ok(())
    }

    /// Delete the directory with the given id from the library. If the directory
    /// doesn't exist, nothing is done.
    fn delete_dir(&mut self, id: &str) -> Result<(), StorageErr> {
        let result = self.find_dir(id);
        let dir = match result {
            Some(dir) => dir,
            None => {
                // does not exist
                debug!("Directory with id '{}' does not exist", id);
                return Ok(());
            }
        };

        // delete the directory itself
        dir.get_dir().delete().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        // delete from memory
        let dirs = self.get_all_dirs_mut();
        dirs.remove(id);

        Ok(())
    }

    /// Delete the directories with the given ids from the library. If the
    /// directory doesn't exist, nothing is done.
    fn delete_dirs(&mut self, ids: &[&String]) -> Result<(), StorageErr> {
        for id in ids {
            self.delete_dir(id)?;
        }
        Ok(())
    }

    /// Delete all directories from the library
    fn delete_all_dirs(&mut self) -> Result<(), StorageErr> {
        self.get_lib_dir()
            .delete()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        self.get_all_dirs_mut().clear();
        self.get_lib_dir()
            .create_if_absent()
            .map_err(|e| StorageErr::FileSysErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(())
    }

    /// Returns the synced directories in the library
    fn get_synced_dirs(&self) -> Vec<&DirT> {
        self.get_all_dirs()
            .iter()
            .filter_map(|(_, dir)| if dir.is_synced() { Some(dir) } else { None })
            .collect()
    }

    /// Returns the manifests of the synced directories in the library
    fn read_synced_mnfs(&self) -> Vec<Arc<ManifestT>> {
        self.get_all_dirs()
            .iter()
            .filter_map(|(_, dir)| {
                if dir.is_synced() {
                    Some(dir.read_mnf())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Returns the unsynced directories in the library
    fn get_unsynced_dirs(&self) -> Vec<&DirT> {
        self.get_all_dirs()
            .iter()
            .filter_map(|(_, dir)| if !dir.is_synced() { Some(dir) } else { None })
            .collect()
    }

    /// Returns the manifests of the unsynced directories in the library
    fn read_unsynced_mnfs(&self) -> Vec<Arc<ManifestT>> {
        self.get_all_dirs()
            .iter()
            .filter_map(|(_, dir)| {
                if !dir.is_synced() {
                    Some(dir.read_mnf())
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_synced(&self, id: &str) -> Result<bool, StorageErr> {
        let dir = self.get_dir(id)?;
        Ok(dir.is_synced())
    }

    /// Sets the synced state of the directory with the given id
    fn set_synced(&mut self, id: &str, synced: bool) -> Result<(), StorageErr> {
        let dir = self.get_dir_mut(id)?;
        dir.set_synced(synced);
        Ok(())
    }

    /// Marks the directory with the given id as synced
    fn mark_synced(&mut self, id: &str) -> Result<(), StorageErr> {
        self.set_synced(id, true)
    }

    /// Checks if all directories in the library are synced
    fn are_all_synced(&self) -> bool {
        let dirs = self.get_all_dirs();
        dirs.is_empty() || dirs.values().all(|dir| dir.is_synced())
    }

    /// Sets the synced state of all directories in the library
    fn set_synced_for_all(&mut self, synced: bool) {
        self.get_all_dirs_mut()
            .iter_mut()
            .for_each(|(_, dir)| dir.set_synced(synced));
    }

    /// Marks all directories in the library as synced
    fn mark_all_synced(&mut self) {
        self.get_all_dirs_mut()
            .iter_mut()
            .for_each(|(_, dir)| dir.mark_synced());
    }
}
