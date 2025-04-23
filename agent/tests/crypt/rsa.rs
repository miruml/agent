#[cfg(test)]
mod tests {
    // internal crates
    use config_agent::crypt::errors::CryptErr;
    use config_agent::crypt::rsa;
    use config_agent::filesys::{dir::Dir, file::File, path::PathExt};

    // external crates
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

    pub mod gen_rsa_key_pair {
        use super::*;

    // TEST CASES
    #[tokio::test]
    async fn success() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());
        private_key_file.delete().await.unwrap();
        public_key_file.delete().await.unwrap();

        let result = rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await;
        assert!(result.is_ok());

        assert!(private_key_file.exists());
        assert!(public_key_file.exists());
    }

    #[tokio::test]
    async fn invalid_key_size() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());

        // Invalid key size
        let result = rsa::gen_rsa_key_pair(0, &private_key_file, &public_key_file).await.unwrap_err();
        assert!(matches!(result, CryptErr::GenerateRSAKeyPairErr { .. }));
    }

    #[tokio::test]
    async fn existing_files() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());
        private_key_file.delete().await.unwrap();
        public_key_file.delete().await.unwrap();

        // public key file exists
        public_key_file.write_bytes(&[4, 4], true, false).await.unwrap();
        let result = rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await;
        assert!(result.is_err());

        // private key file exists
        private_key_file.write_bytes(&[4, 4], true, false).await.unwrap();
        let result = rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await;
        assert!(result.is_err());
    }
}

pub mod read_rsa_private_key_file {
    use super::*;

    #[tokio::test]
    async fn success() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());
        private_key_file.delete().await.unwrap();
        public_key_file.delete().await.unwrap();

        rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await.unwrap();

        let result = rsa::read_rsa_private_key_file(&private_key_file).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invalid_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        private_key_file.delete().await.unwrap();

        private_key_file.write_bytes(&[4, 4], true, false).await.unwrap();
        let result = rsa::read_rsa_private_key_file(&private_key_file).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn missing_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        private_key_file.delete().await.unwrap();

        let result = rsa::read_rsa_private_key_file(&private_key_file).await;
        assert!(result.is_err());
    }

}

pub mod read_rsa_public_key_file {
    use super::*;

    #[tokio::test]
    async fn success() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());
        private_key_file.delete().await.unwrap();
        public_key_file.delete().await.unwrap();

        rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await.unwrap();

        let result = rsa::read_rsa_public_key_file(&public_key_file).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn invalid_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let public_key_file = File::new(public_key_path.clone());
        public_key_file.delete().await.unwrap();

        public_key_file.write_bytes(&[4, 4], true, false).await.unwrap();
        let result = rsa::read_rsa_public_key_file(&public_key_file).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn missing_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let public_key_file = File::new(public_key_path.clone());
        public_key_file.delete().await.unwrap();

        let result = rsa::read_rsa_public_key_file(&public_key_file).await;
        assert!(result.is_err());
    }
}

pub mod create_rsa_signature {
    use super::*;

    #[tokio::test]
    async fn success1() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());
        private_key_file.delete().await.unwrap();
        public_key_file.delete().await.unwrap();

        rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await.unwrap();

        let data = b"hello world";
        let signature = rsa::create_rsa_signature(&private_key_file, data).await.unwrap();
        assert!(!signature.is_empty());
    }

    #[tokio::test]
    async fn invalid_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        private_key_file.delete().await.unwrap();

        private_key_file.write_bytes(&[4, 4], true, false).await.unwrap();
        let data = b"hello world";
        let result = rsa::create_rsa_signature(&private_key_file, data).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn missing_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        private_key_file.delete().await.unwrap();

        let data = b"hello world";
        let result = rsa::create_rsa_signature(&private_key_file, data).await;
        assert!(result.is_err());
    }
}

pub mod verify_rsa_signature {
    use super::*;

    #[tokio::test]
    async fn success() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let private_key_path = crypt_dir.path().join("private_key.pem");
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let private_key_file = File::new(private_key_path.clone());
        let public_key_file = File::new(public_key_path.clone());
        private_key_file.delete().await.unwrap();
        public_key_file.delete().await.unwrap();

        rsa::gen_rsa_key_pair(2048, &private_key_file, &public_key_file).await.unwrap();

        let data = b"hello world";
        let signature = rsa::create_rsa_signature(&private_key_file, data).await.unwrap();
        let result = rsa::verify_rsa_signature(&public_key_file, data, &signature).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn invalid_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let public_key_file = File::new(public_key_path.clone());
        public_key_file.delete().await.unwrap();

        public_key_file.write_bytes(&[4, 4], true, false).await.unwrap();
        let data = b"hello world";
        let signature = vec![4, 4];
        let result = rsa::verify_rsa_signature(&public_key_file, data, &signature).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn missing_file() {
        let crypt_dir = Dir::create_temp_dir("crypt_rsa_test").await.unwrap();
        let public_key_path = crypt_dir.path().join("public_key.pem");

        let public_key_file = File::new(public_key_path.clone());
        public_key_file.delete().await.unwrap();

        let data = b"hello world";
        let signature = vec![4, 4];
        let result = rsa::verify_rsa_signature(&public_key_file, data, &signature).await;
        assert!(result.is_err());
    }
}
}
