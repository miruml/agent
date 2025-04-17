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

    // Create a new Dir instance from the parent directory of the path for this File
    // instance
    pub fn parent(&self) -> Result<Dir, FileSysErr> {
        let parent = self.path
            .parent()
            .ok_or(FileSysErr::UnknownFileParentDirErr {
                file: self.clone(),
                trace: trace!(),
            })?;
        Ok(Dir::new(parent))
    }

    pub fn parent_exists(&self) -> Result<bool, FileSysErr> {
        // check parent directory exists
        let parent = self.parent()?;
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

    fn validate_overwrite(dest: &File, overwrite: bool) -> Result<(), FileSysErr> {
        if !overwrite && dest.exists() {
            return Err(FileSysErr::PathExists {
                path: dest.path().clone(),
                trace: trace!(),
            });
        }
        Ok(())
    }

    /// Write bytes to a file. Overwrites the file if it exists.
    pub fn write_bytes(
        &self,
        buf: &[u8],
        overwrite: bool,
    ) -> Result<(), FileSysErr> {
        // ensure parent directory exists
        let parent = self.parent()?;
        if !parent.exists() {
            parent.create_if_absent()?;
        }

        File::validate_overwrite(self, overwrite)?;

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
    pub fn write_string(&self, s: &str, overwrite: bool) -> Result<(), FileSysErr> {
        self.write_bytes(s.as_bytes(), overwrite)
    }

    /// Write a JSON object to a file. Overwrites the file if it exists.
    pub fn write_json<T: serde::Serialize>(
        &self,
        obj: &T,
        overwrite: bool,
    ) -> Result<(), FileSysErr> {
        File::validate_overwrite(self, overwrite)?;

        // Convert to JSON bytes first
        let json_bytes = serde_json::to_vec(obj).map_err(|e| FileSysErr::ParseJSONErr {
            source: e,
            file: self.clone(),
            trace: trace!(),
        })?;

        self.write_bytes(&json_bytes, overwrite)
    }

    /// Rename this file to a new file. Overwrites the new file if it exists.
    pub fn move_to(&self, new_file: &File, overwrite: bool) -> Result<(), FileSysErr> {
        // source file must exist
        self.assert_exists()?;

        // if this file and the new file are the same, nothing needs to be done
        if self.path() == new_file.path() {
            return Ok(());
        }

        File::validate_overwrite(new_file, overwrite)?;

        // ensure the parent directory of the new file exists and create it if not
        new_file.parent()?.create_if_absent()?;
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
        File::validate_overwrite(link, overwrite)?;
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

    fn metadata(&self) -> Result<std::fs::Metadata, FileSysErr> {
        self.assert_exists()?;
        std::fs::metadata(self.to_string()).map_err(|e| FileSysErr::FileMetaDataErr {
            source: e,
            trace: trace!(),
        })
    }

    pub fn permissions(&self) -> Result<std::fs::Permissions, FileSysErr> {
        Ok(self.metadata()?.permissions())
    }

    pub fn last_modified(&self) -> Result<SystemTime, FileSysErr> {
        Ok(self.metadata()?.modified().unwrap_or(SystemTime::now()))
    }

    pub fn size(&self) -> Result<u64, FileSysErr> {
        Ok(self.metadata()?.len())
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
