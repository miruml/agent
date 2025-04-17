#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt};
    // external crates
    use std::path::PathBuf;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod delete {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod name {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

pub mod move_to {
    use super::*;

    fn success() {
        assert!(false);
    }

    #[test]
    fn same_file() {
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

pub mod sanitize_filename {
    use super::*;

    #[test]
    fn success() {
        assert!(false);
    }
}

}
