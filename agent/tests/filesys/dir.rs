#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    // external crates
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};


pub mod dir_is_valid_dir_name {
    use super::*;

    #[test]
    fn success() {
        let dir_name = "is_valid_dir_name";
        assert!(Dir::is_valid_dir_name(dir_name));
    }

    #[test]
    fn empty_string() {
        let dir_name = "";
        assert!(!Dir::is_valid_dir_name(dir_name));
    }

    #[test]
    fn contains_slash() {
        let dir_name = "invalid/dir_name";
        assert!(!Dir::is_valid_dir_name(dir_name));
    }

    #[test]
    fn contains_null_byte() {
        let dir_name = "invalid\0dir_name";
        assert!(!Dir::is_valid_dir_name(dir_name));
    }

    #[test]
    fn exceeds_max_length() {
        let dir_name = "a".repeat(256); // A string with 256 characters
        assert!(!Dir::is_valid_dir_name(&dir_name));
    }

    #[test]
    fn exact_max_length() {
        let dir_name = "a".repeat(255); // A string with 255 characters
        assert!(Dir::is_valid_dir_name(&dir_name));
    }

    #[test]
    fn contains_special_characters() {
        let special_chars = "!@#$%^&*()";
        for special_char in special_chars.chars() {
            let dir_name = format!("is_valid_dir_name{}", special_char);
            assert!(Dir::is_valid_dir_name(&dir_name));
        }
    }

    #[test]
    fn contains_leading_trailing_spaces() {
        let dir_name = "  is_valid_dir_name  ";
        assert!(Dir::is_valid_dir_name(dir_name));
    }
}

mod delete_if_empty {
    use super::*;

    #[test]
    fn success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        dir.delete_if_empty().unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn has_files() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.new_file("test");
        file.write_string("arglechargle").unwrap();
        dir.delete_if_empty().unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn has_subdirs() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let subdir = dir.subdir(["test"]);
        subdir.create(true).unwrap();
        dir.delete_if_empty().unwrap();
        assert!(dir.exists());
    }

    #[test]
    #[ignore]
    fn sandbox() {
        let dir = Dir::new_home_dir().unwrap();
        let subdir = dir.subdir(["test"]);
        subdir.delete_if_empty().unwrap();
        assert!(!subdir.exists());
    }

}

mod delete_all_modified_before {
    use super::*;

    #[test]
    fn success_deleted() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        let n = 10;
        for i in 0..n {
            let file = dir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        dir.delete_all_modified_before(std::time::Duration::from_millis(1))
            .unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn success_not_deleted() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        let n = 10;
        for i in 0..n {
            let file = dir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }
        dir.delete_all_modified_before(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn success_recursive_deleted() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // parent directory
        let n = 10;
        for i in 0..n {
            let file = dir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        // child directory
        let subdir = dir.subdir(["test"]);
        for i in 0..n {
            let file = subdir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
        dir.delete_all_modified_before(std::time::Duration::from_millis(1))
            .unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn success_recursive_not_deleted1() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // parent directory
        let n = 10;
        for i in 0..n {
            let file = dir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        // child directory
        let subdir = dir.subdir(["test"]);
        for i in 0..n {
            let file = subdir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        dir.delete_all_modified_before(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn success_recursive_not_deleted2() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // parent directory
        let n = 10;
        for i in 0..n {
            let file = dir.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        // child directory 1
        let subdir1 = dir.subdir(["test1"]);
        for i in 0..n {
            let file = subdir1.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(10));

        // child directory 2
        let subdir2 = dir.subdir(["test2"]);
        for i in 0..n {
            let file = subdir2.new_file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        dir.delete_all_modified_before(std::time::Duration::from_millis(10))
            .unwrap();
        assert!(dir.exists());
        assert_eq!(dir.list_files().unwrap().len(), 0);
        assert!(!subdir1.exists());
        assert!(subdir2.exists());
    }
}


pub mod create_subdir {
    use super::*;

    #[test]
    fn success_overwrite() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        let subdir = dir.subdir(["test"]);
        subdir.create_if_absent().unwrap();
        let file = subdir.new_file("test");
        file.write_string("test").unwrap();
        assert!(subdir.exists());

        // overwriting the subdir should delete the subdir and its file
        let subdir = dir.create_subdir(&["test"], true).unwrap();
        assert!(subdir.exists());
        assert!(!file.exists());
    }

    #[test]
    fn success_not_overwrite() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        let subdir = dir.subdir(["test"]);
        subdir.create_if_absent().unwrap();
        let file = subdir.new_file("test");
        file.write_string("test").unwrap();
        assert!(subdir.exists());

        // not overwriting the subdir should not delete the subdir and its file
        let subdir = dir.create_subdir(&["test"], false).unwrap();
        assert!(subdir.exists());
        assert!(file.exists());
    }
}

}
