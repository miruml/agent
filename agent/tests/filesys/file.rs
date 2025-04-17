#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::filesys::{
        dir::Dir,
        file::File,
        path::PathExt,
        errors::FileSysErr,
    };
    // external crates
    use std::path::PathBuf;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod delete {
    use super::*;

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test").unwrap();
        assert!(file.exists());
        file.delete().unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn doesnt_exist() {
        let file = File::new(PathBuf::from("doesnt_exist"));
        assert!(!file.exists());
        file.delete().unwrap();
        assert!(!file.exists());
    }
}

pub mod name {
    use super::*;

    #[test]
    fn basic_names() {
        let file = File::new(PathBuf::from("lebron").join("james.txt"));
        assert_eq!(file.name().unwrap(), "james.txt");

        let file = File::new(PathBuf::from("lebron").join("james.txt").join(""));
        assert_eq!(file.name().unwrap(), "james.txt");
    }

    #[test]
    fn with_special_characters() {
        let file = File::new(PathBuf::from("path").join("my-file_123.txt"));
        assert_eq!(file.name().unwrap(), "my-file_123.txt");

        let file = File::new(PathBuf::from("path").join("file.with.dots.txt")); 
        assert_eq!(file.name().unwrap(), "file.with.dots.txt");

        let file = File::new(PathBuf::from("path").join("file with spaces.txt"));
        assert_eq!(file.name().unwrap(), "file with spaces.txt");
    }

    #[test]
    fn with_unicode() {
        let file = File::new(PathBuf::from("path").join("文件.txt"));
        assert_eq!(file.name().unwrap(), "文件.txt");

        let file = File::new(PathBuf::from("path").join("файл.txt"));
        assert_eq!(file.name().unwrap(), "файл.txt");

        let file = File::new(PathBuf::from("path").join("🦀.txt"));
        assert_eq!(file.name().unwrap(), "🦀.txt");
    }
}

pub mod move_to {
    use super::*;

    #[test]
    fn src_doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");

        // overwrite false
        assert!(matches!(
            file.move_to(&file, false).unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));

        // overwrite true
        assert!(matches!(
            file.move_to(&file, false).unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn dest_doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let src = dir.file("src-file");
        src.write_string("test").unwrap();
        let dest = dir.file("dest-file");

        // overwrite false
        assert!(src.exists());
        assert!(!dest.exists());
        src.move_to(&dest, false).unwrap();
        assert!(dest.exists());
        assert!(!src.exists());

        // overwrite true
        let tmp = src;
        let src = dest;
        let dest = tmp;
        assert!(src.exists());
        assert!(!dest.exists());
        src.move_to(&dest, true).unwrap();
        assert!(dest.exists());
        assert!(!src.exists());
    }

    #[test]
    fn dest_exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let src = dir.file("src-file");
        src.write_string("src").unwrap();
        let dest = dir.file("dest-file");
        dest.write_string("dest").unwrap();

        // overwrite false
        assert!(src.exists());
        assert!(dest.exists());
        assert!(matches!(
            src.move_to(&dest, false).unwrap_err(),
            FileSysErr::PathExists { .. }
        ));
    }

    #[test]
    fn dest_exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let src = dir.file("src-file");
        src.write_string("src").unwrap();
        let dest = dir.file("dest-file");
        dest.write_string("dest").unwrap();

        // overwrite false
        assert!(src.exists());
        assert!(dest.exists());
        src.move_to(&dest, true).unwrap();
        assert!(dest.exists());
        assert!(!src.exists());
    }

    #[test]
    fn src_and_dest_are_same_file() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test").unwrap();
        file.move_to(&file, false).unwrap();
        file.assert_exists().unwrap();
        file.move_to(&file, true).unwrap();
        assert!(file.exists());
        assert!(file.read_string().unwrap() == "test");
    }
}

pub mod copy_to {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod extension {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod assert_extension_is {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod assert_path_contains {  
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod parent_exists {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod read_bytes {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod read_string {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod read_json {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod write_bytes {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}   

pub mod write_string {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod write_json {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod par_dir {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod set_permissions {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod create_symlink {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod metadata {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod last_modified {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod size {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
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
}

pub mod sanitize_filename {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

}
