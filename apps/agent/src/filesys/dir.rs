// standard library
use std::env;
use std::fmt::Display;
use std::path::PathBuf;

// internal crates
use crate::filesys::{
    errors::{
        CreateDirErr, DeleteDirErr, FileSysErr, InvalidDirNameErr, ReadDirErr,
        UnknownCurrentDirErr, UnknownDirNameErr, UnknownHomeDirErr, UnknownParentDirForDirErr,
    },
    file::File,
    path::PathExt,
};
use crate::trace;

// external crates
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
}

impl Dir {
    /// Create a new Dir instance. Dir paths must be absolute but do not need to exist
    /// to create a valid Dir instance.
    pub fn new<T: Into<PathBuf>>(path: T) -> Dir {
        Dir { path: path.into() }
    }

    /// Create a new Dir instance for the home directory
    pub fn new_home_dir() -> Result<Dir, FileSysErr> {
        let home_dir = std::env::var("HOME")
            .map_err(|e| {
                FileSysErr::UnknownHomeDirErr(UnknownHomeDirErr {
                    source: Box::new(e),
                    trace: trace!(),
                })
            })
            .map(PathBuf::from)?;
        Ok(Dir { path: home_dir })
    }

    pub fn new_current_dir() -> Result<Dir, FileSysErr> {
        let current_dir = std::env::current_dir().map_err(|e| {
            FileSysErr::UnknownCurrentDirErr(UnknownCurrentDirErr {
                source: Box::new(e),
                trace: trace!(),
            })
        })?;
        Ok(Dir { path: current_dir })
    }

    pub async fn create_temp_dir(prefix: &str) -> Result<Dir, FileSysErr> {
        let temp_dir = Dir::new(env::temp_dir());
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let subdir_name = format!("{}_{}", prefix, timestamp);
        let temp_dir = temp_dir.subdir(PathBuf::from(subdir_name));
        temp_dir.create(true).await?;
        Ok(temp_dir)
    }

    /// Return the name of the directory
    pub fn name(&self) -> Result<&str, FileSysErr> {
        let file_name_os_str: &std::ffi::OsStr;
        match self.path.file_name() {
            Some(name) => file_name_os_str = name,
            None => {
                return Err(FileSysErr::UnknownDirNameErr(UnknownDirNameErr {
                    dir: self.clone(),
                    trace: trace!(),
                }))
            }
        }
        match file_name_os_str.to_str() {
            Some(name) => Ok(name),
            None => Err(FileSysErr::UnknownDirNameErr(UnknownDirNameErr {
                dir: self.clone(),
                trace: trace!(),
            })),
        }
    }

    pub fn parent(&self) -> Result<Dir, FileSysErr> {
        let abs_path = self.abs_path()?;
        let parent = abs_path
            .parent()
            .ok_or(FileSysErr::UnknownParentDirForDirErr(
                UnknownParentDirForDirErr {
                    dir: self.clone(),
                    trace: trace!(),
                },
            ))?;
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
            return Err(FileSysErr::InvalidDirNameErr(InvalidDirNameErr {
                name: dir_name.to_string(),
                trace: trace!(),
            }));
        }
        Ok(())
    }

    /// Create a new Dir instance using a relative path from the current directory
    pub fn subdir<T: Into<PathBuf>>(&self, rel_path: T) -> Dir {
        let rel_path = rel_path.into();
        let rel_path_stripped = rel_path
            .strip_prefix(std::path::MAIN_SEPARATOR.to_string())
            .unwrap_or(&rel_path);
        let mut new_dir = self.path.clone();
        new_dir = new_dir.join(rel_path_stripped);
        Dir::new(new_dir)
    }

    /// Create a new directory in the filesystem and any missing parent directories at
    /// the specified path of this Dir instance. If the directory already exists, it is
    /// deleted if overwrite is true but an error is thrown if overwrite is false.
    pub async fn create(&self, overwrite: bool) -> Result<(), FileSysErr> {
        if !overwrite {
            self.assert_doesnt_exist()?;
        } else {
            self.delete().await?;
        }
        tokio::fs::create_dir_all(self.to_string())
            .await
            .map_err(|e| {
                FileSysErr::CreateDirErr(CreateDirErr {
                    source: Box::new(e),
                    dir: self.clone(),
                    trace: trace!(),
                })
            })?;
        Ok(())
    }

    /// Delete a directory and all its contents
    pub async fn delete(&self) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        tokio::fs::remove_dir_all(self.path()).await.map_err(|e| {
            FileSysErr::DeleteDirErr(DeleteDirErr {
                source: Box::new(e),
                dir: self.clone(),
                trace: trace!(),
            })
        })?;
        Ok(())
    }

    /// Create a new directory in the filesystem and any missing parent directories at
    /// the specified path of this Dir instance
    pub async fn create_if_absent(&self) -> Result<(), FileSysErr> {
        if self.exists() {
            return Ok(());
        }
        self.create(false).await?;
        Ok(())
    }

    /// Create a new File instance using a filename appended to this directory
    pub fn file(&self, file_name: &str) -> File {
        use std::path::Path;
        let file_name_path = Path::new(file_name);
        let file_name_stripped = file_name_path
            .strip_prefix(std::path::MAIN_SEPARATOR.to_string())
            .unwrap_or(file_name_path);
        let file_path = self.path.join(file_name_stripped);
        File::new(file_path)
    }

    /// Return the subdirectories of this directory
    pub async fn subdirs(&self) -> Result<Vec<Dir>, FileSysErr> {
        let mut dirs = Vec::new();
        let mut entries = tokio::fs::read_dir(self.to_string()).await.map_err(|e| {
            FileSysErr::ReadDirErr(ReadDirErr {
                source: Box::new(e),
                dir: self.clone(),
                trace: trace!(),
            })
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            FileSysErr::ReadDirErr(ReadDirErr {
                source: Box::new(e),
                dir: self.clone(),
                trace: trace!(),
            })
        })? {
            if entry.path().is_dir() {
                let dir = Dir::new(entry.path());
                dir.assert_exists()?;
                dirs.push(dir);
            }
        }
        Ok(dirs)
    }

    // Return the files in this directory
    pub async fn files(&self) -> Result<Vec<File>, FileSysErr> {
        let mut files = Vec::new();

        let mut entries = tokio::fs::read_dir(self.to_string()).await.map_err(|e| {
            FileSysErr::ReadDirErr(ReadDirErr {
                source: Box::new(e),
                dir: self.clone(),
                trace: trace!(),
            })
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            FileSysErr::ReadDirErr(ReadDirErr {
                source: Box::new(e),
                dir: self.clone(),
                trace: trace!(),
            })
        })? {
            if entry.path().is_file() {
                let file = File::new(entry.path());
                file.assert_exists()?;
                files.push(file);
            }
        }
        Ok(files)
    }

    pub async fn delete_if_empty(&self) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        if !self.files().await?.is_empty() {
            return Ok(());
        }
        if !self.subdirs().await?.is_empty() {
            return Ok(());
        }
        self.delete().await?;
        Ok(())
    }
}
