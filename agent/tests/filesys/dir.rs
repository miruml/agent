#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::filesys::{dir::Dir, path::PathExt, errors::FileSysErr};
    // external crates
    use std::{env, path::PathBuf};
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod delete {
    use super::*;

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        assert!(dir.exists());
        dir.delete().unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn doesnt_exist() {
        let dir = Dir::new(PathBuf::from("doesnt_exist"));
        assert!(!dir.exists());
        dir.delete().unwrap();
        assert!(!dir.exists());
    }
}

pub mod new_home_dir {
    use super::*;

    #[test]
    fn success() {
        let dir = Dir::new_home_dir().unwrap();
        assert!(dir.exists());

        println!("dir: {}", dir.path().to_str().unwrap());
        assert!(dir.path().to_str().unwrap().contains("home"));
    }
}

pub mod create_temp_dir {
    use super::*;

    #[test]
    fn success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        assert!(dir.exists());
        assert!(dir.path().to_str().unwrap().contains("testing"));
    }
}

pub mod name {
    use super::*;

pub mod success {
    use super::*;

    #[test]
    fn basic_names() {
        let dir = Dir::new(PathBuf::from("lebron").join("james"));
        assert_eq!(dir.name().unwrap(), "james");

        let dir = Dir::new(PathBuf::from("lebron").join("james").join(""));
        assert_eq!(dir.name().unwrap(), "james");
    }

    #[test]
    fn with_special_characters() {
        let dir = Dir::new(PathBuf::from("path").join("my-dir_123"));
        assert_eq!(dir.name().unwrap(), "my-dir_123");

        let dir = Dir::new(PathBuf::from("path").join("dir.with.dots"));
        assert_eq!(dir.name().unwrap(), "dir.with.dots");

        let dir = Dir::new(PathBuf::from("path").join("dir with spaces"));
        assert_eq!(dir.name().unwrap(), "dir with spaces");
    }

    #[test]
    fn with_unicode() {
        let dir = Dir::new(PathBuf::from("path").join("目录"));
        assert_eq!(dir.name().unwrap(), "目录");

        let dir = Dir::new(PathBuf::from("path").join("привет"));
        assert_eq!(dir.name().unwrap(), "привет");

        let dir = Dir::new(PathBuf::from("path").join("🦀"));
        assert_eq!(dir.name().unwrap(), "🦀");
    }
}

pub mod failure {
    use super::*;

    #[test]
    fn root_directory() {
        let dir = Dir::new(PathBuf::from("/"));
        assert!(matches!(
            dir.name().unwrap_err(), FileSysErr::NoDirNameErr { .. }
        ));
    }

    #[test]
    fn empty_path() {
        let dir = Dir::new("");
        assert!(matches!(
            dir.name().unwrap_err(), FileSysErr::NoDirNameErr { .. }
        ));
    }
}
}

pub mod parent {
    use super::*;

pub mod success {
    use super::*;

    #[test]
    fn simple() {
        let dir = Dir::new(PathBuf::from("path").join("dir"));
        assert_eq!(dir.parent().unwrap().name().unwrap(), "path");
    }

    #[test]
    fn with_trailing_separator() {
        let dir = Dir::new(PathBuf::from("path").join("dir").join(""));
        assert_eq!(dir.parent().unwrap().name().unwrap(), "path");
    }

    #[test]
    fn with_trailing_separator_and_dot() {
        let dir = Dir::new(PathBuf::from("path").join("dir").join("."));
        assert_eq!(dir.parent().unwrap().name().unwrap(), "path");
    }

    #[test]
    fn with_trailing_separator_and_dot_dot() {
        let dir = Dir::new(PathBuf::from("bronny").join("james").join("jr").join(".."));
        assert_eq!(dir.parent().unwrap().name().unwrap(), "bronny");
    }

    #[test]
    fn empty_path() {
        let dir = Dir::new("");

        let current_dir_path = env::current_dir().unwrap();
        let expected = current_dir_path.parent().unwrap();
        assert_eq!(dir.parent().unwrap().path(), expected);
    }
}

pub mod failure {
    use super::*;

    #[test]
    fn root_directory() {
        let dir = Dir::new(PathBuf::from("/"));
        assert!(matches!(
            dir.parent().unwrap_err(), FileSysErr::UnknownDirParentDirErr { .. }
        ));
    }
}
}

pub mod valid_dir_name {
    use super::*;

pub mod success {
    use super::*;

    #[test]
    fn basic() {
        let dir_name = "is_valid_dir_name";
        assert!(Dir::is_valid_dir_name(dir_name));
        Dir::assert_valid_dir_name(dir_name).unwrap();
    }

    #[test]
    fn exact_max_length() {
        let dir_name = "a".repeat(255); // A string with 255 characters
        assert!(Dir::is_valid_dir_name(&dir_name));
        Dir::assert_valid_dir_name(&dir_name).unwrap();
    }

    #[test]
    fn contains_special_characters() {
        let special_chars = "!@#$%^&*()";
        for special_char in special_chars.chars() {
            let dir_name = format!("is_valid_dir_name{}", special_char);
            assert!(Dir::is_valid_dir_name(&dir_name));
            Dir::assert_valid_dir_name(&dir_name).unwrap();
        }
    }

    #[test]
    fn contains_leading_trailing_spaces() {
        let dir_name = "  is_valid_dir_name  ";
        assert!(Dir::is_valid_dir_name(dir_name));
        Dir::assert_valid_dir_name(dir_name).unwrap();
    }
}

pub mod failure {
    use super::*;

    #[test]
    fn empty_string() {
        let dir_name = "";
        assert!(!Dir::is_valid_dir_name(dir_name));
        assert!(matches!(
            Dir::assert_valid_dir_name(dir_name).unwrap_err(), FileSysErr::InvalidDirNameErr { .. }
        ));
    }

    #[test]
    fn contains_slash() {
        let dir_name = "invalid/dir_name";
        assert!(!Dir::is_valid_dir_name(dir_name));
        assert!(matches!(
            Dir::assert_valid_dir_name(dir_name).unwrap_err(), FileSysErr::InvalidDirNameErr { .. }
        ));
    }

    #[test]
    fn contains_null_byte() {
        let dir_name = "invalid\0dir_name";
        assert!(!Dir::is_valid_dir_name(dir_name));
        assert!(matches!(
            Dir::assert_valid_dir_name(dir_name).unwrap_err(), FileSysErr::InvalidDirNameErr { .. }
        ));
    }

    #[test]
    fn exceeds_max_length() {
        let dir_name = "a".repeat(256); // A string with 256 characters
        assert!(!Dir::is_valid_dir_name(&dir_name));
        assert!(matches!(
            Dir::assert_valid_dir_name(&dir_name).unwrap_err(), FileSysErr::InvalidDirNameErr { .. }
        ));
    }
}
}

mod subdir {
    use super::*;

    #[test]
    fn basic() {
        let dir = Dir::new(PathBuf::from("path").join("dir"));
        let subdir = dir.subdir(PathBuf::from("subdir"));
        assert_eq!(subdir.path(), &dir.path().join("subdir"));
        assert_eq!(subdir.name().unwrap(), "subdir");
    }

    #[test]
    fn nested_subdirs() {
        let base_path = PathBuf::from("base").join("path");
        let base_dir = Dir::new(base_path);
        let nested = base_dir
            .subdir("level1")
            .subdir("level2")
            .subdir("level3");
        
        let expected_path = PathBuf::from("base")
            .join("path")
            .join("level1")
            .join("level2")
            .join("level3");
        assert_eq!(nested.path(), &expected_path);
        assert_eq!(nested.name().unwrap(), "level3");
    }

    #[test]
    fn with_spaces() {
        let dir = Dir::new(PathBuf::from("test"));
        let subdir = dir.subdir("space folder");
        assert_eq!(subdir.path(), &PathBuf::from("test").join("space folder"));
        assert_eq!(subdir.name().unwrap(), "space folder");

        let subdir = dir.subdir("hyphen-folder");
        assert_eq!(subdir.path(), &PathBuf::from("test").join("hyphen-folder"));
        assert_eq!(subdir.name().unwrap(), "hyphen-folder");
    }

    #[test]
    fn with_empty_path() {
        let dir = Dir::new(PathBuf::from("test"));
        let subdir = dir.subdir("");
        assert_eq!(subdir.path(), &PathBuf::from("test").join(""));
    }

    #[test]
    fn with_absolute_path_component() {
        let dir = Dir::new(PathBuf::from("test"));
        let path_component = PathBuf::from("absolute").join("path");
        let subdir = dir.subdir(path_component);
        assert_eq!(
            subdir.path(), 
            &PathBuf::from("test").join("absolute").join("path")
        );
    }

    #[test]
    fn with_dot_paths() {
        let dir = Dir::new(PathBuf::from("test"));
        let subdir = dir.subdir(".");
        assert_eq!(subdir.path(), &PathBuf::from("test").join("."));

        let subdir = dir.subdir("..");
        assert_eq!(subdir.path(), &PathBuf::from("test").join(".."));
    }
}

mod create {
    use super::*;

mod success {
    use super::*;

    #[test]
    fn doesnt_exist_with_overwrite() {
        let temp_dir= Dir::create_temp_dir("testing").unwrap();

        let subdir = temp_dir.subdir(PathBuf::from("subdir"));
        subdir.create(true).unwrap();
        assert!(subdir.exists());
    }

    #[test]
    fn doesnt_exist_no_overwrite() {
        let temp_dir= Dir::create_temp_dir("testing").unwrap();

        let subdir = temp_dir.subdir(PathBuf::from("subdir"));
        subdir.create(false).unwrap();
        assert!(subdir.exists());
    }

    #[test]
    fn exists_with_overwrite() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        dir.create(true).unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn exists_no_overwrite() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        println!("ERROR: {:?}", dir.create(false).unwrap_err());
        assert!(matches!(
            dir.create(false).unwrap_err(), FileSysErr::PathExists { .. }
        ));
    }
}
}

mod create_if_absent {
    use super::*;

    #[test]
    fn doesnt_exist() {
        let temp_dir= Dir::create_temp_dir("testing").unwrap();

        let subdir = temp_dir.subdir(PathBuf::from("subdir"));
        subdir.create_if_absent().unwrap();
        assert!(subdir.exists());
    }

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // create some files in the directory to check if they exist afterward
        let file = dir.file("test-file");
        file.write_string("arglebargle").unwrap();

        // create the directory
        dir.create_if_absent().unwrap();
        assert!(dir.exists());
        assert!(file.exists());
    }
}

mod file {
    use super::*;

    #[test]
    fn abs_dir() {
        let dir_path = PathBuf::from("tmp").join("test_dir");
        let dir = Dir::new(dir_path.clone());
        let file = dir.file("test.txt");
        assert_eq!(file.path(), &dir_path.join("test.txt"));
    }

    #[test]
    fn nested_file_path() {
        let dir_path = PathBuf::from("base").join("test_dir");
        let dir = Dir::new(dir_path.clone());
        let file = dir.file("nested/folder/test.txt");
        assert_eq!(file.path(), &dir_path.join("nested").join("folder").join("test.txt"));
    }

    #[test]
    fn special_characters() {
        let dir_path = PathBuf::from("test_dir");
        let dir = Dir::new(dir_path.clone());
        
        // Test spaces in filename
        let file = dir.file("my file.txt");
        assert_eq!(file.path(), &dir_path.join("my file.txt"));
        
        // Test dots in filename
        let file = dir.file("test.multiple.dots.txt");
        assert_eq!(file.path(), &dir_path.join("test.multiple.dots.txt"));
        
        // Test hyphens and underscores
        let file = dir.file("my-file_name.txt");
        assert_eq!(file.path(), &dir_path.join("my-file_name.txt"));
    }

    #[test]
    fn empty_filename() {
        let dir_path = PathBuf::from("test_dir");
        let dir = Dir::new(dir_path.clone());
        let file = dir.file("");
        assert_eq!(file.path(), &dir_path.join(""));
    }

    #[test]
    fn with_different_extensions() {
        let dir_path = PathBuf::from("test_dir");
        let dir = Dir::new(dir_path.clone());
        
        // No extension
        let file = dir.file("filename");
        assert_eq!(file.path(), &dir_path.join("filename"));
        
        // Common extensions
        let file = dir.file("image.png");
        assert_eq!(file.path(), &dir_path.join("image.png"));
        
        // Hidden file (Unix-style)
        let file = dir.file(".hidden");
        assert_eq!(file.path(), &dir_path.join(".hidden"));
    }

    #[test]
    fn with_unicode_filename() {
        let dir_path = PathBuf::from("test_dir");
        let dir = Dir::new(dir_path.clone());
        
        let file = dir.file("文件.txt");
        assert_eq!(file.path(), &dir_path.join("文件.txt"));
        
        let file = dir.file("🦀rust.rs");
        assert_eq!(file.path(), &dir_path.join("🦀rust.rs"));
    }
}

mod subdirs {
    use super::*;

    #[test]
    fn empty() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        assert_eq!(dir.subdirs().unwrap().len(), 0);
    }

    #[test]
    fn success() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // create some subdirs
        let subdir1 = dir.subdir(PathBuf::from("subdir1"));
        subdir1.create(true).unwrap();
        let subdir2 = dir.subdir(PathBuf::from("subdir2"));
        subdir2.create(true).unwrap();
        assert!(subdir1.exists());
        assert!(subdir2.exists());

        // get the subdirs
        let subdirs = dir.subdirs().unwrap();
        assert_eq!(subdirs.len(), 2);
        assert!(subdirs.iter().any(|d| d.path() == subdir1.path()));
        assert!(subdirs.iter().any(|d| d.path() == subdir2.path()));
    }
}

mod files {
    use super::*;

    #[test]
    fn empty() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        assert_eq!(dir.files().unwrap().len(), 0);
    }

    #[test]
    fn success() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // create some files
        let file1 = dir.file("file1.txt");
        file1.write_string("arglebargle").unwrap();
        let file2 = dir.file("file2.txt");
        file2.write_string("arglebargle").unwrap();

        // get the files
        let files = dir.files().unwrap();
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.path() == file1.path()));
        assert!(files.iter().any(|f| f.path() == file2.path()));
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
        let file = dir.file("test");
        file.write_string("arglechargle").unwrap();
        dir.delete_if_empty().unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn has_subdirs() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let subdir = dir.subdir(PathBuf::from("test"));
        subdir.create(true).unwrap();
        dir.delete_if_empty().unwrap();
        assert!(dir.exists());
    }
}

mod delete_contents_modified_before {
    use super::*;

    #[test]
    fn success_deleted() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        let n = 10;
        for i in 0..n {
            let file = dir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
        dir.delete_contents_modified_before(std::time::Duration::from_millis(1))
            .unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn success_not_deleted() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        let n = 10;
        for i in 0..n {
            let file = dir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }
        dir.delete_contents_modified_before(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn success_recursive_deleted() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // parent directory
        let n = 10;
        for i in 0..n {
            let file = dir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        // child directory
        let subdir = dir.subdir(PathBuf::from("test"));
        for i in 0..n {
            let file = subdir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
        dir.delete_contents_modified_before(std::time::Duration::from_millis(1))
            .unwrap();
        assert!(!dir.exists());
    }

    #[test]
    fn success_recursive_not_deleted1() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // parent directory
        let n = 10;
        for i in 0..n {
            let file = dir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        // child directory
        let subdir = dir.subdir(PathBuf::from("test"));
        for i in 0..n {
            let file = subdir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        dir.delete_contents_modified_before(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(dir.exists());
    }

    #[test]
    fn success_recursive_not_deleted2() {
        let dir = Dir::create_temp_dir("testing").unwrap();

        // parent directory
        let n = 10;
        for i in 0..n {
            let file = dir.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        // child directory 1
        let subdir1 = dir.subdir(PathBuf::from("test1"));
        for i in 0..n {
            let file = subdir1.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(10));

        // child directory 2
        let subdir2 = dir.subdir(PathBuf::from("test2"));
        for i in 0..n {
            let file = subdir2.file(&format!("test-file-{}", i));
            file.write_string("test").unwrap();
        }

        dir.delete_contents_modified_before(std::time::Duration::from_millis(10))
            .unwrap();
        assert!(dir.exists());
        assert_eq!(dir.files().unwrap().len(), 0);
        assert!(!subdir1.exists());
        assert!(subdir2.exists());
    }
}
}
