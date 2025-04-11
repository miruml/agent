// standard library
use std::path::PathBuf;
// internal crates
use crate::filesys::errors::FileSysErr;
use crate::trace;

pub trait PathExt {
    fn path(&self) -> &PathBuf;

    fn delete(&self) -> Result<(), FileSysErr>;

    fn exists(&self) -> bool {
        self.path().exists()
    }

    fn assert_exists(&self) -> Result<(), FileSysErr> {
        if !self.exists() {
            return Err(FileSysErr::PathDoesNotExist {
                path: self.path().clone(),
                trace: trace!(),
            });
        }
        Ok(())
    }

    fn assert_doesnt_exist(&self) -> Result<(), FileSysErr> {
        if self.exists() {
            return Err(FileSysErr::PathExists {
                path: self.path().clone(),
                trace: trace!(),
            });
        }
        Ok(())
    }
}
