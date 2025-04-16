// standard library
use std::fmt::Display;
use std::fs;
use std::io::BufReader;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
// internal crates
use crate::filesys::dir::Dir;
use crate::filesys::errors::FileSysErr;
use crate::filesys::path::PathExt;
use crate::trace;
// external crates
use serde::de::DeserializeOwned;
#[allow(unused_imports)]
use tracing::{debug, error, info, warn};

// DEFINITIONS
/// File struct for interacting with files
#[derive(Clone, Debug)]
pub struct File {
    path: PathBuf,
}

impl Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_str().unwrap_or_default())
    }
}

impl PathExt for File {
    fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Delete a file
    fn delete(&self) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        fs::remove_file(self.to_string()).map_err(|e| FileSysErr::DeleteFileErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;
        Ok(())
    }
}

impl File {
    /// Create a new File instance
    pub fn new<T: Into<PathBuf>>(path: T) -> Self {
        let _path: PathBuf = path.into();
        if _path.is_relative() {
            warn!(
                "Path '{}' must be an absolute path",
                _path.to_str().unwrap_or("")
            );
        }
        File { path: _path }
    }

    /// Return the name of the file
    pub fn name(&self) -> Result<&str, FileSysErr> {
        let file_name_os_str: &std::ffi::OsStr;
        match self.path.file_name() {
            Some(name) => file_name_os_str = name,
            None => {
                return Err(FileSysErr::NoFileNameErr {
                    file: self.clone(),
                    trace: trace!(),
                })
            }
        }
        match file_name_os_str.to_str() {
            Some(name) => Ok(name),
            None => Err(FileSysErr::NoFileNameErr {
                file: self.clone(),
                trace: trace!(),
            }),
        }
    }

    /// Rename this file to a new file. Overwrites the new file if it exists.
    pub fn move_to(&self, new_file: &File, overwrite: bool) -> Result<(), FileSysErr> {
        // source file must exist
        self.assert_exists()?;

        // if this file and the new file are the same, nothing needs to be done
        if self.path() == new_file.path() {
            return Ok(());
        }

        // check not overwriting new file if it exist
        if !overwrite && new_file.exists() {
            return Err(FileSysErr::PathExists {
                path: new_file.path().clone(),
                trace: trace!(),
            });
        }

        // ensure the parent directory of the new file exists and create it if not
        new_file.par_dir()?.create_if_absent()?;
        if overwrite {
            new_file.delete()?;
        }

        // move this file to the new file
        std::fs::rename(self.to_string(), new_file.to_string()).map_err(|e| {
            FileSysErr::MoveFileErr {
                source: e,
                src_file: self.clone(),
                dest_file: new_file.clone(),
                trace: trace!(),
            }
        })?;
        Ok(())
    }

    /// Copy this file to a new file. Overwrites the new file if it exists.
    pub fn copy_to(&self, new_file: &File, overwrite: bool) -> Result<(), FileSysErr> {
        // source file must exist
        self.assert_exists()?;

        // if this file and the new file are the same, nothing needs to be done
        if self.path() == new_file.path() {
            return Ok(());
        }

        // check not overwriting new file if it exist
        if !overwrite && new_file.exists() {
            return Err(FileSysErr::PathExists {
                path: new_file.path().clone(),
                trace: trace!(),
            });
        }

        // ensure the parent directory of the new file exists and create it if not
        new_file.par_dir()?.create_if_absent()?;
        if overwrite {
            new_file.delete()?;
        }

        std::fs::copy(self.to_string(), new_file.to_string()).map_err(|e| {
            FileSysErr::CopyFileErr {
                source: e,
                src_file: self.clone(),
                dest_file: new_file.clone(),
                trace: trace!(),
            }
        })?;
        Ok(())
    }

    /// Return's the file path's extension
    pub fn extension(&self) -> Option<&str> {
        self.path
            .extension()
            .map(|ext| ext.to_str().unwrap_or_default())
    }

    /// Check if the file has the given extension. Returns an error if not.
    pub fn assert_extension_is(&self, ext: &str) -> Result<(), FileSysErr> {
        if !self.extension().unwrap_or_default().eq(ext) {
            return Err(FileSysErr::BadFileExtensionErr {
                expected: ext.to_string(),
                actual: self.extension().unwrap_or_default().to_string(),
                file: self.clone(),
                trace: trace!(),
            });
        }
        Ok(())
    }

    pub fn assert_path_contains(&self, expr: &str) -> Result<(), FileSysErr> {
        if !self.path().to_str().unwrap_or_default().contains(expr) {
            return Err(FileSysErr::BadFilePathExprErr {
                expected: expr.to_string(),
                actual: self.extension().unwrap_or_default().to_string(),
                file: self.clone(),
                trace: trace!(),
            });
        }
        Ok(())
    }

    pub fn parent_exists(&self) -> Result<bool, FileSysErr> {
        // check parent directory exists
        let parent = self.par_dir()?;
        Ok(parent.exists())
    }

    /// Read the contents of a file
    pub fn read_bytes(&self) -> Result<Vec<u8>, FileSysErr> {
        self.assert_exists()?;

        // read file
        let mut file = fs::File::open(self.to_string()).map_err(|e| FileSysErr::OpenFileErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| FileSysErr::ReadFileErr {
                source: e,
                file: self.clone(),
                trace: trace!(),
            })?;
        Ok(buf)
    }

    /// Read the contents of a file as a string
    pub fn read_string(&self) -> Result<String, FileSysErr> {
        let bytes = self.read_bytes()?;
        let str_ = std::str::from_utf8(&bytes).map_err(|e| FileSysErr::ConvertUTF8Err {
            source: e,
            trace: trace!(),
        })?;
        Ok(str_.to_string())
    }

    /// Read the contents of a file as json
    pub fn read_json<T: DeserializeOwned>(&self) -> Result<T, FileSysErr> {
        self.assert_exists()?;
        self.assert_path_contains(".json")?;

        // read file
        let file = fs::File::open(self.to_string()).map_err(|e| FileSysErr::OpenFileErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;
        let reader = BufReader::new(file);
        let obj: T = serde_json::from_reader(reader).map_err(|e| FileSysErr::ParseJSONErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;
        Ok(obj)
    }

    /// Write bytes to a file. Overwrites the file if it exists.
    pub fn write_bytes(&self, buf: &[u8]) -> Result<(), FileSysErr> {
        // ensure parent directory exists
        let par_dir = self.par_dir()?;
        if !par_dir.exists() {
            par_dir.create_if_absent()?;
        }

        let mut file =
            fs::File::create(self.to_string()).map_err(|e| FileSysErr::OpenFileErr {
                source: e,
                file: self.clone(),
                trace: trace!(),
            })?;
        file.write_all(buf).map_err(|e| FileSysErr::WriteFileErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;

        Ok(())
    }

    /// Write a string to a file. Overwrites the file if it exists.
    pub fn write_string(&self, s: &str) -> Result<(), FileSysErr> {
        self.write_bytes(s.as_bytes())
    }

    /// Write a JSON object to a file. Overwrites the file if it exists.
    pub fn write_json<T: serde::Serialize>(
        &self,
        obj: &T,
        overwrite: bool,
    ) -> Result<(), FileSysErr> {
        self.assert_path_contains(".json")?;

        // if file exists and overwrite is false, return an error
        if !overwrite && self.exists() {
            return Err(FileSysErr::PathExists {
                path: self.path().clone(),
                trace: trace!(),
            });
        }

        // Convert to JSON bytes first
        let json_bytes = serde_json::to_vec(obj).map_err(|e| FileSysErr::ParseJSONErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;

        self.write_bytes(&json_bytes)
    }

    // Create a new Dir instance from the parent directory of the path for this File
    // instance
    pub fn par_dir(&self) -> Result<Dir, FileSysErr> {
        let parent = self
            .path
            .parent()
            .ok_or(FileSysErr::UnknownFileParentDirErr {
                file: self.clone(),
                trace: trace!(),
            })?;
        Ok(Dir::new(parent))
    }

    // Set the file permissions using octal
    // (https://www.redhat.com/sysadmin/linux-file-permissions-explained)
    pub fn set_permissions(&self, mode: u32) -> Result<(), FileSysErr> {
        self.assert_exists()?;

        // set file permissions
        fs::set_permissions(self.to_string(), fs::Permissions::from_mode(mode)).map_err(|e| {
            FileSysErr::WriteFileErr {
                source: e,
                file: self.clone(),
                trace: trace!(),
            }
        })?;
        Ok(())
    }

    // overwrites a symlink if it already exists
    pub fn create_symlink(&self, link: &File, overwrite: bool) -> Result<(), FileSysErr> {
        self.assert_exists()?;
        if !overwrite && link.exists() {
            return Err(FileSysErr::PathExists {
                path: link.path().clone(),
                trace: trace!(),
            });
        }
        link.delete()?;

        // create symlink
        std::os::unix::fs::symlink(self.to_string(), link.to_string()).map_err(|e| {
            FileSysErr::CreateSymlinkErr {
                source: e,
                file: self.clone(),
                link: link.clone(),
                trace: trace!(),
            }
        })?;
        Ok(())
    }

    pub fn metadata(&self) -> Result<std::fs::Metadata, FileSysErr> {
        self.assert_exists()?;
        std::fs::metadata(self.to_string()).map_err(|e| FileSysErr::FileMetaDataErr {
            source: e,
            trace: trace!(),
        })
    }

    pub fn last_modified(&self) -> Result<SystemTime, FileSysErr> {
        self.metadata()
            .map(|m| m.modified().unwrap_or(SystemTime::now()))
    }

    pub fn size(&self) -> Result<u64, FileSysErr> {
        self.metadata().map(|m| m.len())
    }

    pub fn delete_if_modified_before(&self, ago: Duration) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Ok(());
        }
        let modified_at = self.last_modified()?;
        let duration_since_modified =
            SystemTime::now().duration_since(modified_at).map_err(|e| {
                FileSysErr::SystemTimeErr {
                    source: e,
                    trace: trace!(),
                }
            })?;
        if duration_since_modified > ago {
            self.delete()?;
        }
        Ok(())
    }
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            // Allow alphanumeric and some safe characters
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            // Replace everything else with underscore
            _ => '_'
        })
        .collect()
}
