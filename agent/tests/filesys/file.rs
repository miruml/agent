#[cfg(test)]
mod tests {
    // standard library
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    // internal crates
    use config_agent::filesys::{
        dir::Dir,
        file,
        file::File,
        path::PathExt,
        errors::FileSysErr,
    };

    // external crates
    use serde_json::json;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod delete {
    use super::*;

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
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
        src.write_string("test", false, false).unwrap();
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
        src.write_string("src", false, false).unwrap();
        let dest = dir.file("dest-file");
        dest.write_string("dest", false, false).unwrap();

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
        src.write_string("src", false, false).unwrap();
        let dest = dir.file("dest-file");
        dest.write_string("dest", false, false).unwrap();

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
        file.write_string("test", false, false).unwrap();
        file.move_to(&file, false).unwrap();
        file.assert_exists().unwrap();
        file.move_to(&file, true).unwrap();
        assert!(file.exists());
        assert!(file.read_string().unwrap() == "test");
    }
}

pub mod parent_exists {
    use super::*;

    #[test]
    fn exists() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        assert!(file.parent_exists().unwrap());
    }

    #[test]
    fn doesnt_exist() {
        let file = File::new(PathBuf::from("doesnt_exist").join("test-file.json"));
        assert!(!file.parent_exists().unwrap());
    }
}

pub mod read_bytes {
    use super::*;

    #[test]
    fn read_doesnt_exist() {
        let file = File::new(PathBuf::from("doesnt_exist").join("test-file.json"));
        assert!(matches!(
            file.read_bytes().unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn read_success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("arglebargle", false, false).unwrap();
        assert_eq!(file.read_bytes().unwrap(), b"arglebargle");
    }
}

pub mod read_string {
    use super::*;

    #[test]
    fn read_doesnt_exist() {
        let file = File::new(PathBuf::from("doesnt_exist").join("test-file.json"));
        assert!(matches!(
            file.read_string().unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn read_success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("arglebargle", false, false).unwrap();
        assert_eq!(file.read_string().unwrap(), "arglebargle");
    }
}

pub mod read_json {
    use super::*;

    #[test]
    fn read_doesnt_exist() {
        let file = File::new(PathBuf::from("doesnt_exist").join("test-file.json"));
        assert!(matches!(
            file.read_json::<String>().unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn read_success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("{\"test\": \"arglebargle\"}", false, false).unwrap();
        assert_eq!(file.read_json::<serde_json::Value>().unwrap(), serde_json::json!({"test": "arglebargle"}));
    }
}

pub mod write_bytes {
    use super::*;

    fn write_bytes_atomic(file: &File, buf: &[u8], overwrite: bool) -> Result<(), FileSysErr> {
        file.write_bytes(buf, overwrite, true)
    }
    fn write_bytes_non_atomic(file: &File, buf: &[u8], overwrite: bool) -> Result<(), FileSysErr> {
        file.write_bytes(buf, overwrite, false)
    }

    #[test]
    fn doesnt_exist() {
        for write_bytes in [write_bytes_atomic, write_bytes_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            write_bytes(&file, b"arglebargle", false).unwrap();
            assert_eq!(file.read_bytes().unwrap(), b"arglebargle");
        }
    }

    #[test]
    fn parent_doesnt_exist  () {
        for write_bytes in [write_bytes_atomic, write_bytes_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let subdir = dir.subdir(PathBuf::from("nested").join("subdir"));
            let file = subdir.file("test-file");
            write_bytes(&file, b"arglebargle", false).unwrap();
            assert_eq!(file.read_bytes().unwrap(), b"arglebargle");
        }
    }

    #[test]
    fn exists_overwrite_false() {
        for write_bytes in [write_bytes_atomic, write_bytes_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            file.write_bytes(b"arglebargle", false, false).unwrap();
            assert_eq!(file.read_bytes().unwrap(), b"arglebargle");

        // should fail when writing again
            assert!(matches!(
                write_bytes(&file, b"arglebargle", false).unwrap_err(),
                FileSysErr::PathExists { .. }
            ));
        }
    }

    #[test]
    fn exists_overwrite_true() {
        for write_bytes in [write_bytes_atomic, write_bytes_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            file.write_bytes(b"arglebargle", false, false).unwrap();
            assert_eq!(file.read_bytes().unwrap(), b"arglebargle");

            // should succeed when writing again
            write_bytes(&file, b"arglebargle", true).unwrap();
            assert_eq!(file.read_bytes().unwrap(), b"arglebargle");
        }
    }
}   

pub mod write_string {
    use super::*;

    fn write_string_atomic(file: &File, s: &str, overwrite: bool) -> Result<(), FileSysErr> {
        file.write_string(s, overwrite, true)
    }
    fn write_string_non_atomic(file: &File, s: &str, overwrite: bool) -> Result<(), FileSysErr> {
        file.write_string(s, overwrite, false)
    }

    #[test]
    fn doesnt_exist() {
        for write_string in [write_string_atomic, write_string_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            write_string(&file, "hello world", false).unwrap();
            assert_eq!(file.read_string().unwrap(), "hello world");
        }
    }

    #[test]
    fn parent_doesnt_exist() {
        for write_string in [write_string_atomic, write_string_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let subdir = dir.subdir(PathBuf::from("nested").join("subdir"));
            let file = subdir.file("test-file");
            write_string(&file, "hello world", false).unwrap();
            assert_eq!(file.read_string().unwrap(), "hello world");
        }
    }

    #[test]
    fn exists_overwrite_false() {
        for write_string in [write_string_atomic, write_string_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            file.write_string("hello world", false, false).unwrap();
            assert_eq!(file.read_string().unwrap(), "hello world");

            // should fail when writing again
            assert!(matches!(
                write_string(&file, "new content", false).unwrap_err(),
                FileSysErr::PathExists { .. }
            ));
        }
    }

    #[test]
    fn exists_overwrite_true() {
        for write_string in [write_string_atomic, write_string_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            file.write_string("hello world", false, false).unwrap();
            assert_eq!(file.read_string().unwrap(), "hello world");

            // should succeed when writing again
            write_string(&file, "new content", true).unwrap();
            assert_eq!(file.read_string().unwrap(), "new content");
        }
    }
}

mod write_json {
    use super::*;

    fn write_json_atomic(file: &File, data: &serde_json::Value, overwrite: bool) -> Result<(), FileSysErr> {
        file.write_json(data, overwrite, true)
    }
    fn write_json_non_atomic(file: &File, data: &serde_json::Value, overwrite: bool) -> Result<(), FileSysErr> {
        file.write_json(data, overwrite, false)
    }

    #[test]
    fn doesnt_exist() {
        for write_json in [write_json_atomic, write_json_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            let data = json!({
                "name": "test",
                "value": 42
                });
            write_json(&file, &data, false).unwrap();
            let read_data: serde_json::Value = file.read_json().unwrap();
            assert_eq!(read_data, data);
        }
    }

    #[test]
    fn parent_doesnt_exist() {
        for write_json in [write_json_atomic, write_json_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let subdir = dir.subdir(PathBuf::from("nested").join("subdir"));
            let file = subdir.file("test-file");
            let data = json!({
            "name": "test",
            "value": 42
            });
            write_json(&file, &data, false).unwrap();
            let read_data: serde_json::Value = file.read_json().unwrap();
            assert_eq!(read_data, data);
        }
    }

    #[test]
    fn exists_overwrite_false() {
        for write_json in [write_json_atomic, write_json_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            let data = json!({
            "name": "test",
            "value": 42
            });
            write_json(&file, &data, false).unwrap();
            let read_data: serde_json::Value = file.read_json().unwrap();
            assert_eq!(read_data, data);

            // should fail when writing again
            let new_data = json!({
                "name": "updated",
                "value": 100
            });
            assert!(matches!(
                write_json(&file, &new_data, false).unwrap_err(),
                FileSysErr::PathExists { .. }
            ));
        }
    }

    #[test]
    fn exists_overwrite_true() {
        for write_json in [write_json_atomic, write_json_non_atomic] {
            let dir = Dir::create_temp_dir("testing").unwrap();
            let file = dir.file("test-file");
            let data = json!({
            "name": "test",
            "value": 42
            });
            write_json(&file, &data, false).unwrap();
            let read_data: serde_json::Value = file.read_json().unwrap();
            assert_eq!(read_data, data);

            // should succeed when writing again
            let new_data = json!({
                "name": "updated",
                "value": 100
            });
            write_json(&file, &new_data, true).unwrap();
            let read_data: serde_json::Value = file.read_json().unwrap();
            assert_eq!(read_data, new_data);
        }
    }
}

pub mod set_permissions {
    use super::*;

    #[test]
    fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("nonexistent-file");
        
        // Should fail because file doesn't exist
        assert!(matches!(
            file.set_permissions(0o644).unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn basic_permissions() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        
        // Create the file first
        file.write_string("test content", false, false).unwrap();
        
        // Test read-only (444 in octal)
        file.set_permissions(0o444).unwrap();
        let perms = file.permissions().unwrap();
        assert_eq!(perms.mode() & 0o777, 0o444);
        
        // Test read-write (644 in octal)
        file.set_permissions(0o644).unwrap();
        let perms = file.permissions().unwrap();
        assert_eq!(perms.mode() & 0o777, 0o644);
        
        // Test executable (755 in octal)
        file.set_permissions(0o755).unwrap();
        let perms = file.permissions().unwrap();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }

    #[cfg(unix)]
    #[test]
    fn all_permission_combinations() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test content", false, false).unwrap();

        // Test various permission combinations
        let permissions = [
            0o400, // read only for owner
            0o200, // write only for owner
            0o100, // execute only for owner
            0o440, // read for owner and group
            0o444, // read for owner, group, and others
            0o666, // read-write for all
            0o777, // read-write-execute for all
        ];

        for mode in permissions {
            file.set_permissions(mode).unwrap();
            let perms = file.permissions().unwrap();
            assert_eq!(perms.mode() & 0o777, mode);
        }
    }
}

pub mod create_symlink {
    use super::*;

    #[test]
    fn src_doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("nonexistent-file");
        let link = dir.file("link");
        assert!(matches!(
            file.create_symlink(&link, false).unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn dest_doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        let link = dir.file("link");

        // overwrite false
        file.create_symlink(&link, false).unwrap();
        file.assert_exists().unwrap();
        link.assert_exists().unwrap();
    }

    #[test]
    fn dest_doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        let link = dir.file("link");

        // overwrite true
        file.create_symlink(&link, true).unwrap();
        file.assert_exists().unwrap();
        link.assert_exists().unwrap();
    }

    #[test]
    fn dest_exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        let link = dir.file("link");
        file.create_symlink(&link, true).unwrap();

        file.assert_exists().unwrap();
        link.assert_exists().unwrap();
        assert!(matches!(
            file.create_symlink(&link, false).unwrap_err(),
            FileSysErr::PathExists { .. }
        ));
    }

    #[test]
    fn dest_exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        let link = dir.file("link");
        file.create_symlink(&link, true).unwrap();

        file.assert_exists().unwrap();
        link.assert_exists().unwrap();
        file.create_symlink(&link, true).unwrap();
        file.assert_exists().unwrap();
        link.assert_exists().unwrap();
    }

}

// permissions test above
pub mod permissions {
    use super::*;

    #[test]
    fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("nonexistent-file");
        
        // Should fail because file doesn't exist
        assert!(matches!(
            file.permissions().unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn basic_permissions() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        
        // Create the file first
        file.write_string("test content", false, false).unwrap();
        
        // Test read-only (444 in octal)
        file.set_permissions(0o444).unwrap();
        let perms = file.permissions().unwrap();
        assert_eq!(perms.mode() & 0o777, 0o444);
        
        // Test read-write (644 in octal)
        file.set_permissions(0o644).unwrap();
        let perms = file.permissions().unwrap();
        assert_eq!(perms.mode() & 0o777, 0o644);
        
        // Test executable (755 in octal)
        file.set_permissions(0o755).unwrap();
        let perms = file.permissions().unwrap();
        assert_eq!(perms.mode() & 0o777, 0o755);
    }
}

pub mod last_modified {
    use super::*;

    #[test]
    fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("nonexistent-file");
        
        // Should fail because file doesn't exist
        assert!(matches!(
            file.last_modified().unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        let modified = file.last_modified().unwrap();
        assert!(modified.elapsed().unwrap() < std::time::Duration::from_secs(1));
    }
}

pub mod size {
    use super::*;

    #[test]
    fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("nonexistent-file");
        
        // Should fail because file doesn't exist
        assert!(matches!(
            file.size().unwrap_err(),
            FileSysErr::PathDoesNotExist { .. }
        ));
    }

    #[test]
    fn success() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        assert_eq!(file.size().unwrap(), 4);
    }
}


pub mod delete_if_modified_before {
    use super::*;

    #[test]
    fn delete_if_modified_before_success_modified() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        file.delete_if_modified_before(std::time::Duration::from_millis(1))
            .unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn delete_if_modified_before_success_not_modified() {
        let dir = Dir::create_temp_dir("testing").unwrap();
        let file = dir.file("test-file");
        file.write_string("test", false, false).unwrap();
        file.delete_if_modified_before(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(file.exists());
    }
}

mod sanitize_filename {
    use super::*;

    #[test]
    fn allowed_characters() {
        // alphabets
        assert_eq!(file::sanitize_filename("abcxyzABCXYZ"), "abcxyzABCXYZ");
        
        // numbers
        assert_eq!(file::sanitize_filename("0123456789"), "0123456789");
        
        // allowed special characters
        assert_eq!(file::sanitize_filename("test-file_name.txt"), "test-file_name.txt");
        
        // mixed allowed characters
        assert_eq!(file::sanitize_filename("File-123_TEST.txt"), "File-123_TEST.txt");
    }

    #[test]
    fn disallowed_characters() {
        // spaces
        assert_eq!(file::sanitize_filename("file name"), "file_name");
        
        // special characters
        assert_eq!(file::sanitize_filename("file@#$%^&*"), "file_______");
        
        // slashes
        assert_eq!(file::sanitize_filename("path/to/file"), "path_to_file");
        assert_eq!(file::sanitize_filename("path\\to\\file"), "path_to_file");
        
        // mixed special characters
        assert_eq!(file::sanitize_filename("my<>file:*?.txt"), "my__file___.txt");
    }

    #[test]
    fn unicode_characters() {
        // emoji
        assert_eq!(file::sanitize_filename("hello😊world"), "hello_world");
        
        // accented characters
        assert_eq!(file::sanitize_filename("résumé.pdf"), "r_sum_.pdf");
        
        // non-Latin scripts
        assert_eq!(file::sanitize_filename("文件.txt"), "__.txt");
        assert_eq!(file::sanitize_filename("файл.txt"), "____.txt");
    }

    #[test]
    fn edge_cases() {
        // empty string
        assert_eq!(file::sanitize_filename(""), "");
        
        // string with only special characters
        assert_eq!(file::sanitize_filename("@#$%^&*"), "_______");
        
        // string with only allowed special characters
        assert_eq!(file::sanitize_filename(".-_"), ".-_");
        
        // repeated special characters
        assert_eq!(file::sanitize_filename("file!!!name"), "file___name");
        
        // leading/trailing special characters
        assert_eq!(file::sanitize_filename("...file..."), "...file...");
        assert_eq!(file::sanitize_filename("###file###"), "___file___");
    }

    #[test]
    fn common_filename_patterns() {
        // common file extensions
        assert_eq!(file::sanitize_filename("document.pdf"), "document.pdf");
        assert_eq!(file::sanitize_filename("image.jpg"), "image.jpg");
        assert_eq!(file::sanitize_filename("script.sh"), "script.sh");
        
        // hidden files (Unix-style)
        assert_eq!(file::sanitize_filename(".gitignore"), ".gitignore");
        
        // version numbers
        assert_eq!(file::sanitize_filename("file-v1.2.3.txt"), "file-v1.2.3.txt");
        
        // common naming patterns
        assert_eq!(file::sanitize_filename("2023-01-01_backup.tar.gz"), "2023-01-01_backup.tar.gz");
        assert_eq!(file::sanitize_filename("file (1)"), "file__1_");
        assert_eq!(file::sanitize_filename("my_file [v2]"), "my_file__v2_");
    }
}

}
