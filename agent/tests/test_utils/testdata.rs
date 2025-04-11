use config_agent::filesys::dir::Dir;

// test file locations
pub fn testdata_dir() -> Dir {
    let project_root_path = env!("CARGO_MANIFEST_DIR");
    let miru_dir = Dir::new(project_root_path);
    miru_dir.parent_dir().unwrap().subdir(&["testdata"])
}

pub fn filesys_testdata_dir() -> Dir {
    let project_root_path = env!("CARGO_MANIFEST_DIR");
    let miru_dir = Dir::new(project_root_path);
    miru_dir.parent_dir().unwrap().subdir(&["testdata"])
}

