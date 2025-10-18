use miru_agent::filesys::dir::Dir;
use std::path::PathBuf;

// test file locations
pub fn testdata_dir() -> Dir {
    let project_root_path = env!("CARGO_MANIFEST_DIR");
    let miru_dir = Dir::new(project_root_path);
    miru_dir.parent().unwrap().subdir(PathBuf::from("testdata"))
}

pub fn filesys_testdata_dir() -> Dir {
    testdata_dir().subdir(PathBuf::from("filesys"))
}

pub fn sandbox_testdata_dir() -> Dir {
    testdata_dir().subdir(PathBuf::from("sandbox"))
}

pub fn crypt_testdata_dir() -> Dir {
    testdata_dir().subdir(PathBuf::from("crypt"))
}
