#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    // external crates
    use std::path::PathBuf;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

    // #[test]
    // fn file_path() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_delete() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_new() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_name() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_move_to() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_copy_to() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_new_temp_file() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_extension() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_assert_extension_is() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_parent_dir_exists() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_read_bytes() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_read_string() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_read_json() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_write_bytes() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_write_string() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_write_json() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_write_http_response() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_unzip() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_par_dir() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_set_permissions() {
    //     assert!(false)
    // }

    // #[test]
    // fn file_create_symlink() {
    //     assert!(false)
    // }

    #[test]
    fn move_to_same_file() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test").unwrap();
        file.move_to(&file, false).unwrap();
        file.assert_exists().unwrap();
        file.move_to(&file, true).unwrap();
        assert!(file.exists());
        assert!(file.read_string().unwrap() == "test");
    }

    #[test]
    fn file_path_success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        assert!(file
            .path()
            .to_str()
            .unwrap_or_default()
            .contains("test-file"));
    }

    #[test]
    fn file_path_parent_success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        assert!(file
            .par_dir()
            .unwrap()
            .path()
            .to_str()
            .unwrap_or_default()
            .contains("testing"));
    }

pub mod delete_if_modified_before {
    use super::*;

    #[test]
    fn delete_if_modified_before_success_modified() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        file.delete_if_modified_before(std::time::Duration::from_millis(1))
            .unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn delete_if_modified_before_success_not_modified() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test").unwrap();
        file.delete_if_modified_before(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(file.exists());
    }

    #[test]
    #[ignore]
    fn delete_if_modified_before_sandbox() {
        let dir = Dir::new_home_dir().unwrap();
        let subdir = dir.subdir(PathBuf::from("test"));
        let file = subdir.file("test.txt");
        file.delete_if_modified_before(std::time::Duration::from_secs(90))
            .unwrap();
        assert!(!file.exists());
    }
}

}
