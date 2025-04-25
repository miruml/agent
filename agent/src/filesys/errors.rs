// standard library
use std::fmt;
use std::path::PathBuf;

// internal crates
use crate::errors::{Code, HTTPCode, MiruError, Trace};
use crate::filesys::{dir::Dir, file::File};

#[derive(Debug)]
pub struct UnknownHomeDirErr {
    pub source: std::env::VarError,
    pub trace: Box<Trace>,
}

impl MiruError for UnknownHomeDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for UnknownHomeDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unable to find home directory: {}", self.source)
    }
}

#[derive(Debug)]
pub struct InvalidDirNameErr {
    pub name: String,
    pub trace: Box<Trace>,
}

impl MiruError for InvalidDirNameErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for InvalidDirNameErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid directory name: {}", self.name)
    }
}

#[derive(Debug)]
pub struct UnknownDirNameErr {
    pub dir: Dir,
    pub trace: Box<Trace>,
}

impl MiruError for UnknownDirNameErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for UnknownDirNameErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to determine directory name for directory path: {}",
            self.dir
        )
    }
}

#[derive(Debug)]
pub struct UnknownFileNameErr {
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for UnknownFileNameErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for UnknownFileNameErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to determine file name for file path: {}",
            self.file
        )
    }
}

#[derive(Debug)]
pub struct PathDoesNotExistErr {
    pub path: PathBuf,
    pub trace: Box<Trace>,
}

impl MiruError for PathDoesNotExistErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for PathDoesNotExistErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "path does not exist: {}",
            self.path.to_str().unwrap_or("unknown")
        )
    }
}

#[derive(Debug)]
pub struct PathExistsErr {
    pub path: PathBuf,
    pub trace: Box<Trace>,
}

impl MiruError for PathExistsErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for PathExistsErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "path exists: {}",
            self.path.to_str().unwrap_or("unknown")
        )
    }
}

#[derive(Debug)]
pub struct InvalidFileOverwriteErr {
    pub file: File,
    pub overwrite: bool,
    pub trace: Box<Trace>,
}

impl MiruError for InvalidFileOverwriteErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for InvalidFileOverwriteErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot overwrite existing file (allow overwrite is {}): {}",
            self.overwrite, self.file
        )
    }
}

#[derive(Debug)]
pub struct UnknownParentDirForDirErr {
    pub dir: Dir,
    pub trace: Box<Trace>,
}

impl MiruError for UnknownParentDirForDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for UnknownParentDirForDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to determine parent directory for directory: {}",
            self.dir
        )
    }
}

#[derive(Debug)]
pub struct UnknownParentDirForFileErr {
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for UnknownParentDirForFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for UnknownParentDirForFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unable to determine parent directory for file: {}",
            self.file
        )
    }
}

#[derive(Debug)]
pub struct ReadDirErr {
    pub dir: Dir,
    pub source: std::io::Error,
    pub trace: Box<Trace>,
}

impl MiruError for ReadDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ReadDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to read directory: {}", self.dir)
    }
}

#[derive(Debug)]
pub struct AtomicWriteFileErr {
    pub file: File,
    pub source: std::io::Error,
    pub trace: Box<Trace>,
}

impl MiruError for AtomicWriteFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for AtomicWriteFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to write file atomically: {}", self.file)
    }
}

#[derive(Debug)]
pub struct ConvertUTF8Err {
    pub source: std::str::Utf8Error,
    pub trace: Box<Trace>,
}

impl MiruError for ConvertUTF8Err {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ConvertUTF8Err {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UTF-8 conversion failed: {}", self.source)
    }
}

#[derive(Debug)]
pub struct CopyFileErr {
    pub source: std::io::Error,
    pub src_file: File,
    pub dest_file: File,
    pub trace: Box<Trace>,
}

impl MiruError for CopyFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for CopyFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to copy file '{}' to '{}': {}",
            self.src_file, self.dest_file, self.source
        )
    }
}

#[derive(Debug)]
pub struct CreateDirErr {
    pub source: std::io::Error,
    pub dir: Dir,
    pub trace: Box<Trace>,
}

impl MiruError for CreateDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for CreateDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to create directory '{}': {}",
            self.dir, self.source
        )
    }
}

#[derive(Debug)]
pub struct CreateSymlinkErr {
    pub source: std::io::Error,
    pub file: File,
    pub link: File,
    pub trace: Box<Trace>,
}

impl MiruError for CreateSymlinkErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for CreateSymlinkErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to create symlink '{}' aliasing to file '{}': {}",
            self.link, self.file, self.source
        )
    }
}

#[derive(Debug)]
pub struct CreateTmpDirErr {
    pub source: std::io::Error,
    pub trace: Box<Trace>,
}

impl MiruError for CreateTmpDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for CreateTmpDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to create temporary directory: {}", self.source)
    }
}

#[derive(Debug)]
pub struct DeleteDirErr {
    pub source: std::io::Error,
    pub dir: Dir,
    pub trace: Box<Trace>,
}

impl MiruError for DeleteDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for DeleteDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to delete directory '{}': {}",
            self.dir, self.source
        )
    }
}

#[derive(Debug)]
pub struct DeleteFileErr {
    pub source: std::io::Error,
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for DeleteFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for DeleteFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to delete file '{}': {}", self.file, self.source)
    }
}

#[derive(Debug)]
pub struct FileMetadataErr {
    pub file: File,
    pub source: std::io::Error,
    pub trace: Box<Trace>,
}

impl MiruError for FileMetadataErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for FileMetadataErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to extract file metadata for file '{}': {}",
            self.file, self.source
        )
    }
}

#[derive(Debug)]
pub struct MoveFileErr {
    pub source: std::io::Error,
    pub src_file: File,
    pub dest_file: File,
    pub trace: Box<Trace>,
}

impl MiruError for MoveFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for MoveFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to move file '{}' to '{}': {}",
            self.src_file, self.dest_file, self.source
        )
    }
}

#[derive(Debug)]
pub struct OpenFileErr {
    pub source: std::io::Error,
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for OpenFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for OpenFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to open file '{}': {}", self.file, self.source)
    }
}

#[derive(Debug)]
pub struct ParseJSONErr {
    pub source: serde_json::Error,
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for ParseJSONErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ParseJSONErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to parse JSON for file '{}': {}",
            self.file, self.source
        )
    }
}

#[derive(Debug)]
pub struct ReadFileErr {
    pub source: std::io::Error,
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for ReadFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for ReadFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to read file '{}': {}", self.file, self.source)
    }
}

#[derive(Debug)]
pub struct UnknownCurrentDirErr {
    pub source: std::io::Error,
    pub trace: Box<Trace>,
}

impl MiruError for UnknownCurrentDirErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for UnknownCurrentDirErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to determine current directory: {}", self.source)
    }
}

#[derive(Debug)]
pub struct WriteFileErr {
    pub source: std::io::Error,
    pub file: File,
    pub trace: Box<Trace>,
}

impl MiruError for WriteFileErr {
    fn code(&self) -> Code {
        Code::InternalServerError
    }

    fn http_status(&self) -> HTTPCode {
        HTTPCode::INTERNAL_SERVER_ERROR
    }

    fn is_network_connection_error(&self) -> bool {
        false
    }

    fn params(&self) -> Option<serde_json::Value> {
        None
    }
}

impl fmt::Display for WriteFileErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to write to file '{}': {}",
            self.file, self.source
        )
    }
}

#[derive(Debug)]
pub enum FileSysErr {
    InvalidDirNameErr(InvalidDirNameErr),
    UnknownDirNameErr(UnknownDirNameErr),
    InvalidFileOverwriteErr(InvalidFileOverwriteErr),
    UnknownFileNameErr(UnknownFileNameErr),
    PathDoesNotExistErr(PathDoesNotExistErr),
    PathExistsErr(PathExistsErr),
    UnknownParentDirForDirErr(UnknownParentDirForDirErr),
    UnknownParentDirForFileErr(UnknownParentDirForFileErr),

    // internal crate errors

    // external crate errors
    AtomicWriteFileErr(AtomicWriteFileErr),
    ConvertUTF8Err(ConvertUTF8Err),
    CopyFileErr(CopyFileErr),
    CreateDirErr(CreateDirErr),
    CreateSymlinkErr(CreateSymlinkErr),
    CreateTmpDirErr(CreateTmpDirErr),
    DeleteDirErr(DeleteDirErr),
    DeleteFileErr(DeleteFileErr),
    FileMetadataErr(FileMetadataErr),
    MoveFileErr(MoveFileErr),
    OpenFileErr(OpenFileErr),
    ParseJSONErr(ParseJSONErr),
    ReadDirErr(ReadDirErr),
    ReadFileErr(ReadFileErr),
    UnknownCurrentDirErr(UnknownCurrentDirErr),
    UnknownHomeDirErr(UnknownHomeDirErr),
    WriteFileErr(WriteFileErr),
}

macro_rules! forward_error_method {
    ($self:ident, $method:ident $(, $arg:expr)?) => {
        match $self {
            Self::InvalidDirNameErr(e) => e.$method($($arg)?),
            Self::UnknownDirNameErr(e) => e.$method($($arg)?),
            Self::InvalidFileOverwriteErr(e) => e.$method($($arg)?),
            Self::UnknownFileNameErr(e) => e.$method($($arg)?),
            Self::PathDoesNotExistErr(e) => e.$method($($arg)?),
            Self::PathExistsErr(e) => e.$method($($arg)?),
            Self::UnknownParentDirForDirErr(e) => e.$method($($arg)?),
            Self::UnknownParentDirForFileErr(e) => e.$method($($arg)?),
            Self::AtomicWriteFileErr(e) => e.$method($($arg)?),
            Self::ConvertUTF8Err(e) => e.$method($($arg)?),
            Self::CopyFileErr(e) => e.$method($($arg)?),
            Self::CreateDirErr(e) => e.$method($($arg)?),
            Self::CreateSymlinkErr(e) => e.$method($($arg)?),
            Self::CreateTmpDirErr(e) => e.$method($($arg)?),
            Self::DeleteDirErr(e) => e.$method($($arg)?),
            Self::DeleteFileErr(e) => e.$method($($arg)?),
            Self::FileMetadataErr(e) => e.$method($($arg)?),
            Self::MoveFileErr(e) => e.$method($($arg)?),
            Self::OpenFileErr(e) => e.$method($($arg)?),
            Self::ParseJSONErr(e) => e.$method($($arg)?),
            Self::ReadDirErr(e) => e.$method($($arg)?),
            Self::ReadFileErr(e) => e.$method($($arg)?),
            Self::UnknownCurrentDirErr(e) => e.$method($($arg)?),
            Self::UnknownHomeDirErr(e) => e.$method($($arg)?),
            Self::WriteFileErr(e) => e.$method($($arg)?),
        }
    };
}

impl fmt::Display for FileSysErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        forward_error_method!(self, fmt, f)
    }
}

impl MiruError for FileSysErr {
    fn code(&self) -> Code {
        forward_error_method!(self, code)
    }

    fn http_status(&self) -> HTTPCode {
        forward_error_method!(self, http_status)
    }

    fn is_network_connection_error(&self) -> bool {
        forward_error_method!(self, is_network_connection_error)
    }

    fn params(&self) -> Option<serde_json::Value> {
        forward_error_method!(self, params)
    }
}
