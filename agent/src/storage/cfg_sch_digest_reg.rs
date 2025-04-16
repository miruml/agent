// internal crates
use crate::filesys::{
    dir::Dir,
    file,
    file::File,
    path::PathExt,
};
use crate::storage::errors::StorageErr;
use crate::trace;

// external crates
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct ConfigSchemaDigests {
    pub raw: String,
    pub resolved: String,
}

pub struct ConfigSchemaDigestRegistry {
    pub dir: Dir,
}

impl ConfigSchemaDigestRegistry {
    pub fn new(dir: Dir) -> Self {
        Self { dir }
    }

    fn digest_file(&self, raw_digest: &str) -> File {
        let mut filename = format!("{}.json", raw_digest);
        filename = file::sanitize_filename(&filename);
        self.dir.file(&filename)
    }

    pub fn read_resolved_digest(
        &self,
        raw_digest: &str,
    ) -> Result<Option<String>, StorageErr> {
        let digest_file = self.digest_file(raw_digest);
        if !digest_file.exists() {
            return Ok(None);
        }

        let digests = digest_file.read_json::<ConfigSchemaDigests>().map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })?;
        Ok(Some(digests.resolved))
    }

    pub fn insert(
        &self,
        raw_digest: &str,
        resolved_digest: &str,
        overwrite: bool,
    ) -> Result<(), StorageErr> {
        let digests = ConfigSchemaDigests {
            raw: raw_digest.to_string(),
            resolved: resolved_digest.to_string(),
        };

        let digest_file = self.digest_file(raw_digest);
        digest_file.write_json(
            &digests,
            overwrite,
        ).map_err(|e| StorageErr::FileSysErr {
            source: e,
            trace: trace!(),
        })
    }

}