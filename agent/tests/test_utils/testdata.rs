use config_agent::filesys::dir::Dir;
use std::path::PathBuf;

// test file locations
pub fn testdata_dir() -> Dir {
    let project_root_path = env!("CARGO_MANIFEST_DIR");
    let miru_dir = Dir::new(project_root_path);
    miru_dir.parent().unwrap().subdir(PathBuf::from("testdata"))
}

pub fn filesys_testdata_dir() -> Dir {
    let project_root_path = env!("CARGO_MANIFEST_DIR");
    let miru_dir = Dir::new(project_root_path);
    miru_dir.parent().unwrap().subdir(PathBuf::from("testdata"))
}

pub fn sandbox_testdata_dir() -> Dir {
    let project_root_path = env!("CARGO_MANIFEST_DIR");
    let miru_dir = Dir::new(project_root_path);
    miru_dir.parent().unwrap().subdir(PathBuf::from("sandbox"))
}
