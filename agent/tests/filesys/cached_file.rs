// internal crates
use miru_agent::authn::token::{Token, Updates};
use miru_agent::filesys::{
    cached_file::{ConcurrentCachedFile, SingleThreadCachedFile},
    dir::Dir,
    errors::FileSysErr,
    path::PathExt,
};

// external crates
use chrono::{Duration, Utc};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

// ========================= SINGLE THREADED CACHED FILE =========================== //
type SingleThreadTokenFile = SingleThreadCachedFile<Token, Updates>;

pub mod new {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");
        let result = SingleThreadTokenFile::new(file).await;
        assert!(matches!(result, Err(FileSysErr::PathDoesNotExistErr(_))));
    }

    #[tokio::test]
    async fn exists_invalid_data() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        file.write_string("invalid-data", false, false)
            .await
            .unwrap();

        // ensure the contents is correct
        let result = SingleThreadTokenFile::new(file).await;
        assert!(matches!(result, Err(FileSysErr::ParseJSONErr(_))));
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
        let cached_file = SingleThreadTokenFile::new(file).await.unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &token);
    }
}

pub mod new_with_default {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let cached_file = SingleThreadTokenFile::new_with_default(file, Token::default())
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
    }

    #[tokio::test]
    async fn exists_invalid_data() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        file.write_string("invalid-data", false, false)
            .await
            .unwrap();

        let cached_file = SingleThreadTokenFile::new_with_default(file, Token::default())
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
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
        let cached_file = SingleThreadTokenFile::new(file).await.unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &token);
    }
}

pub mod create {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let cached_file = SingleThreadTokenFile::create(file, &Token::default(), false)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
    }

    #[tokio::test]
    async fn doesnt_exist_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let cached_file = SingleThreadTokenFile::create(file, &Token::default(), true)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
    }

    #[tokio::test]
    async fn exists_overwrite_false() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        file.write_string("invalid-data", false, false)
            .await
            .unwrap();

        // should throw an error since already exists
        let result = SingleThreadTokenFile::create(file, &Token::default(), false).await;
        assert!(matches!(
            result,
            Err(FileSysErr::InvalidFileOverwriteErr(_))
        ));
    }

    #[tokio::test]
    async fn exists_overwrite_true() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        file.write_string("invalid-data", false, false)
            .await
            .unwrap();

        // should throw an error since already exists
        let cached_file = SingleThreadTokenFile::create(file, &Token::default(), true)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
    }
}

pub mod read {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let cached_file = SingleThreadTokenFile::create(file, &Token::default(), false)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
    }

    #[tokio::test]
    async fn file_deleted() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        let cached_file = SingleThreadTokenFile::create(file.clone(), &Token::default(), true)
            .await
            .unwrap();

        // delete the file
        file.delete().await.unwrap();
        assert!(!file.exists());

        // should still be able to read the file since it's cached in memory
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());
    }
}

pub mod write {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        let mut cached_file = SingleThreadTokenFile::create(file, &Token::default(), false)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());

        // write to the file
        let token = Token {
            token: "test-token".to_string(),
            expires_at: Utc::now() + Duration::days(1),
        };
        cached_file.write(token.clone()).await.unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &token);
    }

    #[tokio::test]
    async fn file_deleted() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        let mut cached_file = SingleThreadTokenFile::create(file.clone(), &Token::default(), false)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());

        // delete the file
        file.delete().await.unwrap();
        assert!(!file.exists());

        // write to the file
        let token = Token {
            token: "test-token".to_string(),
            expires_at: Utc::now() + Duration::days(1),
        };
        cached_file.write(token.clone()).await.unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &token);
    }
}

pub mod patch {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let mut cached_file = SingleThreadTokenFile::create(file, &Token::default(), false)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());

        // patch the file
        let updates = Updates {
            token: Some("test-token".to_string()),
            expires_at: Some(Utc::now() + Duration::days(1)),
        };
        let expected = Token {
            token: updates.token.clone().unwrap(),
            expires_at: updates.expires_at.unwrap(),
        };
        cached_file.patch(updates).await.unwrap();
        assert_eq!(&expected, cached_file.read().await.as_ref());
    }

    #[tokio::test]
    async fn file_deleted() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let mut cached_file = SingleThreadTokenFile::create(file.clone(), &Token::default(), false)
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.as_ref(), &Token::default());

        // delete the file
        file.delete().await.unwrap();
        assert!(!file.exists());

        // patch the file
        let updates = Updates {
            token: Some("test-token".to_string()),
            expires_at: Some(Utc::now() + Duration::days(1)),
        };
        let expected = Token {
            token: updates.token.clone().unwrap(),
            expires_at: updates.expires_at.unwrap(),
        };
        cached_file.patch(updates).await.unwrap();
        assert_eq!(&expected, cached_file.read().await.as_ref());
    }
}

// ========================= MULTI THREADED CACHED FILE =========================== //
type ConcurrentTokenFile = ConcurrentCachedFile<Token, Updates>;

pub mod spawn {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");
        let result = ConcurrentTokenFile::spawn(64, file).await;
        assert!(matches!(result, Err(FileSysErr::PathDoesNotExistErr(_))));
    }

    #[tokio::test]
    async fn exists_invalid_data() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        file.write_string("invalid-data", false, false)
            .await
            .unwrap();

        let result = ConcurrentTokenFile::spawn(64, file).await;
        assert!(matches!(result, Err(FileSysErr::ParseJSONErr(_))));
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

        let (cached_file, _) = ConcurrentTokenFile::spawn(64, file).await.unwrap();
        assert_eq!(cached_file.read().await.unwrap().as_ref(), &token);
    }
}

pub mod spawn_with_default {
    use super::*;

    #[tokio::test]
    async fn doesnt_exist() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) = ConcurrentTokenFile::spawn_with_default(64, file, Token::default())
            .await
            .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );
    }

    #[tokio::test]
    async fn exists_invalid_data() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        // create the file
        file.write_string("invalid-data", false, false)
            .await
            .unwrap();

        let (cached_file, _) = ConcurrentTokenFile::spawn_with_default(64, file, Token::default())
            .await
            .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );
    }

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let token = Token {
            token: "test-token".to_string(),
            expires_at: Utc::now() + Duration::days(1),
        };
        file.write_json(&token, false, false).await.unwrap();

        let (cached_file, _) = ConcurrentTokenFile::spawn_with_default(64, file, Token::default())
            .await
            .unwrap();
        assert_eq!(cached_file.read().await.unwrap().as_ref(), &token);
    }
}

pub mod shutdown {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let token = Token {
            token: "test-token".to_string(),
            expires_at: Utc::now() + Duration::days(1),
        };
        file.write_json(&token, false, false).await.unwrap();

        let (cached_file, handle) =
            ConcurrentTokenFile::spawn_with_default(64, file, Token::default())
                .await
                .unwrap();
        assert_eq!(cached_file.read().await.unwrap().as_ref(), &token);

        // shutdown the file
        cached_file.shutdown().await.unwrap();
        handle.await.unwrap();
    }
}

pub mod concurrent_read {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) = ConcurrentTokenFile::spawn_with_default(64, file, Token::default())
            .await
            .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );
    }

    #[tokio::test]
    async fn file_deleted() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) =
            ConcurrentTokenFile::spawn_with_default(64, file.clone(), Token::default())
                .await
                .unwrap();

        // delete the file
        file.delete().await.unwrap();
        assert!(!file.exists());

        // should still be able to read the file since it's cached in memory
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );
    }
}

pub mod concurrent_write {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) =
            ConcurrentTokenFile::spawn_with_default(64, file.clone(), Token::default())
                .await
                .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );

        // write to the file
        let token = Token {
            token: "test-token".to_string(),
            expires_at: Utc::now() + Duration::days(1),
        };
        cached_file.write(token.clone()).await.unwrap();
        assert_eq!(cached_file.read().await.unwrap().as_ref(), &token);
    }

    #[tokio::test]
    async fn file_deleted() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) =
            ConcurrentTokenFile::spawn_with_default(64, file.clone(), Token::default())
                .await
                .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );

        // delete the file
        file.delete().await.unwrap();
        assert!(!file.exists());

        // write to the file
        let token = Token {
            token: "test-token".to_string(),
            expires_at: Utc::now() + Duration::days(1),
        };
        cached_file.write(token.clone()).await.unwrap();
        assert_eq!(cached_file.read().await.unwrap().as_ref(), &token);
    }
}

pub mod concurrent_patch {
    use super::*;

    #[tokio::test]
    async fn exists() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) =
            ConcurrentTokenFile::spawn_with_default(64, file.clone(), Token::default())
                .await
                .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );

        // patch the file

        let updates = Updates {
            token: Some("test-token".to_string()),
            expires_at: Some(Utc::now() + Duration::days(1)),
        };
        let expected = Token {
            token: updates.token.clone().unwrap(),
            expires_at: updates.expires_at.unwrap(),
        };
        cached_file.patch(updates).await.unwrap();
        assert_eq!(&expected, cached_file.read().await.unwrap().as_ref());
    }

    #[tokio::test]
    async fn file_deleted() {
        let dir = Dir::create_temp_dir("testing").await.unwrap();
        let file = dir.file("test-file");

        let (cached_file, _) =
            ConcurrentTokenFile::spawn_with_default(64, file.clone(), Token::default())
                .await
                .unwrap();
        assert_eq!(
            cached_file.read().await.unwrap().as_ref(),
            &Token::default()
        );

        // delete the file
        file.delete().await.unwrap();
        assert!(!file.exists());

        // patch the file
        let updates = Updates {
            token: Some("test-token".to_string()),
            expires_at: Some(Utc::now() + Duration::days(1)),
        };
        let expected = Token {
            token: updates.token.clone().unwrap(),
            expires_at: updates.expires_at.unwrap(),
        };
        cached_file.patch(updates).await.unwrap();
        assert_eq!(&expected, cached_file.read().await.unwrap().as_ref());
    }
}
