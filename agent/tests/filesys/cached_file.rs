#[cfg(test)]
mod tests {

    // internal crates
    use config_agent::storage::token::Token;
    use config_agent::filesys::{
        cached_file::CachedFile,
        dir::Dir,
        errors::FileSysErr,
        path::PathExt,
    };

    // external crates
    use chrono::{Utc, Duration};
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

    pub mod new {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");
            let result = CachedFile::<Token>::new(file).await;
            assert!(
                matches!(result, Err(FileSysErr::PathDoesNotExistErr(_))));
        }

        #[tokio::test]
        async fn exists_invalid_data() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            file.write_string("invalid-data", false, false).await.unwrap();

            // ensure the contents is correct
            let result = CachedFile::<Token>::new(file).await;
            assert!(
                matches!(result, Err(FileSysErr::ParseJSONErr(_))));
        }

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            let token = Token {
                token: "test-token".to_string(),
                expires_at: Utc::now() + Duration::days(1),
            };
            file.write_json(&token, false, false).await.unwrap();

            // ensure the contents is correct
            let cached_file = CachedFile::<Token>::new(file).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &token);
        }
    }

    pub mod new_with_default {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            let cached_file = CachedFile::<Token>::new_with_default(file, Token::default()).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }

        #[tokio::test]
        async fn exists_invalid_data() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            file.write_string("invalid-data", false, false).await.unwrap();

            let cached_file = CachedFile::<Token>::new_with_default(file, Token::default()).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            let token = Token {
                token: "test-token".to_string(),
                expires_at: Utc::now() + Duration::days(1),
            };
            file.write_json(&token, false, false).await.unwrap();

            // ensure the contents is correct
            let cached_file = CachedFile::<Token>::new(file).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &token);
        }
    }

    pub mod create {
        use super::*;

        #[tokio::test]
        async fn doesnt_exist_overwrite_false() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            let cached_file = CachedFile::<Token>::create(file, &Token::default(), false).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }

        #[tokio::test]
        async fn doesnt_exist_overwrite_true() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            let cached_file = CachedFile::<Token>::create(file, &Token::default(), true).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }

        #[tokio::test]
        async fn exists_overwrite_false() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            file.write_string("invalid-data", false, false).await.unwrap();

            // should throw an error since already exists
            let result = CachedFile::<Token>::create(file, &Token::default(), false).await;
            assert!(matches!(result, Err(FileSysErr::InvalidFileOverwriteErr(_))));
        }

        #[tokio::test]
        async fn exists_overwrite_true() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            file.write_string("invalid-data", false, false).await.unwrap();

            // should throw an error since already exists
            let cached_file = CachedFile::<Token>::create(file, &Token::default(), true).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }
    }

    pub mod read {
        use super::*;

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            let cached_file = CachedFile::<Token>::create(file, &Token::default(), false).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }

        #[tokio::test]
        async fn file_deleted() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            let cached_file = CachedFile::<Token>::create(file.clone(), &Token::default(), true).await.unwrap();

            // delete the file
            file.delete().await.unwrap();
            assert!(!file.exists());

            // should still be able to read the file since it's cached in memory
            assert_eq!(cached_file.read().as_ref(), &Token::default());
        }
    }

    pub mod write {
        use super::*;

        #[tokio::test]
        async fn exists() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            let mut cached_file = CachedFile::<Token>::create(file, &Token::default(), false).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());

            // write to the file
            let token = Token {
                token: "test-token".to_string(),
                expires_at: Utc::now() + Duration::days(1),
            };
            cached_file.write(token.clone()).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &token);
        }

        #[tokio::test]
        async fn file_deleted() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let file = dir.file("test-file");

            // create the file
            let mut cached_file = CachedFile::<Token>::create(file.clone(), &Token::default(), false).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &Token::default());

            // delete the file
            file.delete().await.unwrap();
            assert!(!file.exists());

            // write to the file
            let token = Token {
                token: "test-token".to_string(),
                expires_at: Utc::now() + Duration::days(1),
            };
            cached_file.write(token.clone()).await.unwrap();
            assert_eq!(cached_file.read().as_ref(), &token);
        }
    }
}