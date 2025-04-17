// standard library
use std::fmt::Display;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

// internal crates
use crate::filesys::errors::FileSysErr;
use crate::filesys::file::File;
use crate::filesys::path::PathExt;
use crate::trace;

#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

/// Dir struct for interacting with directories
#[derive(Clone, Debug)]
pub struct Dir {
    path: PathBuf,
}

impl Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_str().unwrap_or_default())
    }
}

impl PathExt for Dir {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Delete a directory and all its contents
    fn delete(&self) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        std::fs::remove_dir_all(self.path()).map_err(|e| FileSysErr::DeleteDirErr {
            source: e,
            dir: self.clone(),
            trace: trace!(),
        })?;
        Ok(())
    }
}

impl Dir {
    /// Create a new Dir instance. Dir paths must be absolute but do not need to exist
    /// to create a valid Dir instance.
    pub fn new<T: Into<PathBuf>>(path: T) -> Dir {
        let _path: PathBuf = path.into();
        if _path.is_relative() {
            warn!(
                "Path '{}' must be an absolute path",
                _path.to_str().unwrap_or("")
            );
        }
        Dir { path: _path }
    }

    /// Create a new Dir instance for the home directory
    pub fn new_home_dir() -> Result<Dir, FileSysErr> {
        let home_dir = std::env::var("HOME")
            .map_err(|_| FileSysErr::CreateHomeDirErr)
            .map(PathBuf::from)?;
        Ok(Dir { path: home_dir })
    }

    pub fn new_current_dir() -> Result<Dir, FileSysErr> {
        let current_dir = std::env::current_dir()
            .map_err(|e| FileSysErr::UnknownCurrentDirErr {
                source: e,
                trace: trace!(),
            })?;
        Ok(Dir { path: current_dir })
    }

    pub fn create_temp_dir(prefix: &str) -> Result<Dir, FileSysErr> {
        let temp_dir = Dir::new(env::temp_dir());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let subdir_name = format!("{}_{}", prefix, timestamp);
        let temp_dir = temp_dir.subdir(PathBuf::from(subdir_name));
        temp_dir.create(true)?;
        Ok(temp_dir)
    }

    /// Return the name of the directory
    pub fn name(&self) -> Result<&str, FileSysErr> {
        let file_name_os_str: &std::ffi::OsStr;
        match self.path.file_name() {
            Some(name) => file_name_os_str = name,
            None => {
                return Err(FileSysErr::NoDirNameErr {
                    dir: self.clone(),
                    trace: trace!(),
                })
            }
        }
        match file_name_os_str.to_str() {
            Some(name) => Ok(name),
            None => Err(FileSysErr::NoDirNameErr {
                dir: self.clone(),
                trace: trace!(),
            }),
        }
    }

    pub fn parent(&self) -> Result<Dir, FileSysErr> {
        let abs_path = self.abs_path()?;
        let parent = abs_path
            .parent()
            .ok_or(FileSysErr::UnknownDirParentDirErr {
                dir: self.clone(),
                trace: trace!(),
            })?;
        Ok(Dir::new(parent))
    }

    pub fn is_valid_dir_name(dir_name: &str) -> bool {
        // Check if the name is empty
        if dir_name.is_empty() {
            return false;
        }

        // Check if the name contains forbidden characters
        if dir_name.contains('/') || dir_name.contains('\0') {
            return false;
        }

        // Check if the name is within the allowed length (255 characters)
        if dir_name.len() > 255 {
            return false;
        }

        true
    }

    pub fn assert_valid_dir_name(dir_name: &str) -> Result<(), FileSysErr> {
        if !Dir::is_valid_dir_name(dir_name) {
            return Err(FileSysErr::InvalidDirNameErr {
                name: dir_name.to_string(),
                trace: trace!(),
            });
        }
        Ok(())
    }

    /// Create a new Dir instance using a relative path from the current directory
    pub fn subdir<T: Into<PathBuf>>(&self, rel_path: T) -> Dir {
        let mut new_dir = self.path.clone();
        new_dir = new_dir.join(rel_path.into());
        Dir::new(new_dir)
    }

    /// Create a new directory in the filesystem and any missing parent directories at
    /// the specified path of this Dir instance. If the directory already exists, it is
    /// deleted if overwrite is true but an error is thrown if overwrite is false.
    pub fn create(&self, overwrite: bool) -> Result<(), FileSysErr> {
        if !overwrite {
            self.assert_doesnt_exist()?;
        } else {
            self.delete()?;
        }
        std::fs::create_dir_all(self.to_string()).map_err(|e| FileSysErr::CreateDirErr {
            source: e,
            dir: self.clone(),
            trace: trace!(),
        })?;
        Ok(())
    }

    /// Create a new directory in the filesystem and any missing parent directories at
    /// the specified path of this Dir instance
    pub fn create_if_absent(&self) -> Result<(), FileSysErr> {
        if self.exists() {
            return Ok(());
        }
        self.create(false)?;
        Ok(())
    }

    /// Create a new File instance using a filename appended to this directory
    pub fn file(&self, file_name: &str) -> File {
        let file_path = self.path.join(file_name);
        File::new(file_path)
    }

    /// Return the subdirectories of this directory
    pub fn subdirs(&self) -> Result<Vec<Dir>, FileSysErr> {
        let mut dirs = Vec::new();
        for entry in std::fs::read_dir(self.to_string()).map_err(|e| FileSysErr::ReadDirErr {
            source: e,
            dir: self.clone(),
            trace: trace!(),
        })? {
            let entry = entry.map_err(|e| FileSysErr::ReadDirErr {
                source: e,
                dir: self.clone(),
                trace: trace!(),
            })?;
            if entry.path().is_dir() {
                let dir = Dir::new(entry.path());
                dir.assert_exists()?;
                dirs.push(dir);
            }
        }
        Ok(dirs)
    }

    // Return the files in this directory
    pub fn files(&self) -> Result<Vec<File>, FileSysErr> {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(self.to_string()).map_err(|e| FileSysErr::ReadDirErr {
            source: e,
            dir: self.clone(),
            trace: trace!(),
        })? {
            let entry = entry.map_err(|e| FileSysErr::ReadDirErr {
                source: e,
                dir: self.clone(),
                trace: trace!(),
            })?;
            if entry.path().is_file() {
                let file = File::new(entry.path());
                file.assert_exists()?;
                files.push(file);
            }
        }
        Ok(files)
    }

    pub fn delete_if_empty(&self) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        if !self.files()?.is_empty() {
            return Ok(());
        }
        if !self.subdirs()?.is_empty() {
            return Ok(());
        }
        self.delete()?;
        Ok(())
    }

    /// Recursively deletes all contents of this directory and its subdirectories which
    /// were modified before the given duration
    pub fn delete_contents_modified_before(&self, ago: Duration) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        for subdir in self.subdirs()? {
            subdir.delete_contents_modified_before(ago)?;
        }
        for file in self.files()? {
            file.delete_if_modified_before(ago)?;
        }
        self.delete_if_empty()?;
        Ok(())
    }
}
