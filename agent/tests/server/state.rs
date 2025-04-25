#[cfg(test)]
mod tests {
    // std crates
    use std::sync::atomic::Ordering;

    // internal crates
    use config_agent::filesys::dir::Dir;
    use config_agent::storage::{
        agent::Agent,
        layout::StorageLayout,
        token::Token,
    };
    use config_agent::server::state::ServerState;
    use config_agent::server::errors::ServerErr;
    use config_agent::filesys::errors::FileSysErr;

    // external crates
    use chrono::Utc;

    pub mod new {
        use super::*;

        #[tokio::test]
        async fn fail_missing_private_key_file() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let layout = StorageLayout::new(dir);
            let result = ServerState::new(layout).await;
            match result {
                Err(ServerErr::FileSysErr(e)) => {
                    assert!(matches!(*e.source, FileSysErr::PathDoesNotExistErr(_)));
                }
                Err(e) => {
                    panic!("Expected FileSysErr not {:?}", e);
                }
                Ok(_) => {
                    panic!("expected error from initializing server state");
                }
            }
        }

        #[tokio::test]
        async fn fail_missing_client_id() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let layout = StorageLayout::new(dir);
            // create a private key file
            let private_key_file = layout.auth_dir().private_key_file();
            private_key_file.write_string("test", false, false).await.unwrap();

            let result = ServerState::new(layout).await;
            assert!(matches!(result, Err(ServerErr::MissingClientIDErr(_))));
        }

        #[tokio::test]
        async fn success_missing_agent_file_but_valid_token() {
            let begin_test = Utc::now().timestamp();
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let layout = StorageLayout::new(dir);

            // create a private key file
            let private_key_file = layout.auth_dir().private_key_file();
            private_key_file.write_string("test", false, false).await.unwrap();

            // create the token file with a token containing a client id
            let token_file = layout.auth_dir().token_file();
            let token = Token {
                token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3NDU2MzgzMTUsInN1YiI6ImNsaV8xMjMiLCJpc3MiOiJtaXJ1IiwiYXVkIjoiY2xpZW50IiwiZXhwIjoxNzIxNTE3MDM0fQ.4ARFzYZSF_i9PjPZRJtH7HcmE_vv5tuZIpKkniua6BY".to_string(),
                expires_at: Utc::now(),
            };
            token_file.write_json(&token, false, false).await.unwrap();

            let (state, _) = ServerState::new(layout.clone()).await.unwrap();

            // the agent file should not have the client id
            let agent_file = layout.agent_file();
            let agent = agent_file.read_json::<Agent>().await.unwrap();
            assert_eq!(agent.client_id, "cli_123");

            // check last activity
            assert!(state.last_activity.load(Ordering::Relaxed) <= Utc::now().timestamp() as u64);
            assert!(state.last_activity.load(Ordering::Relaxed) >= begin_test as u64);
        }

        #[tokio::test]
        async fn success_missing_token_file() {
            let begin_test = Utc::now().timestamp();
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let layout = StorageLayout::new(dir);

            // create a private key file
            let private_key_file = layout.auth_dir().private_key_file();
            private_key_file.write_string("test", false, false).await.unwrap();

            // create the agent file
            let agent_file = layout.agent_file();
            let agent = Agent {
                client_id: "cli_123".to_string(),
            };
            agent_file.write_json(&agent, false, false).await.unwrap();

            let (state, _) = ServerState::new(layout.clone()).await.unwrap();

            // the token file should now have the default token
            let token_file = layout.auth_dir().token_file();
            let token = token_file.read_json::<Token>().await.unwrap();
            assert_eq!(token.token, Token::default().token);

            // check last activity
            assert!(state.last_activity.load(Ordering::Relaxed) <= Utc::now().timestamp() as u64);
            assert!(state.last_activity.load(Ordering::Relaxed) >= begin_test as u64);
        }
    }

    pub mod shutdown {
        use super::*;

        #[tokio::test]
        async fn success() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let layout = StorageLayout::new(dir);

            // create a private key file
            let private_key_file = layout.auth_dir().private_key_file();
            private_key_file.write_string("test", false, false).await.unwrap();

            // create the agent file
            let agent_file = layout.agent_file();
            let agent = Agent {
                client_id: "cli_123".to_string(),
            };
            agent_file.write_json(&agent, false, false).await.unwrap();

            let (state, state_handle) = ServerState::new(layout.clone()).await.unwrap();
            state.shutdown().await.unwrap();
            state_handle.await;
        }
    }

    pub mod record_activity {
        use super::*;

        #[tokio::test]
        async fn success_record_activity() {
            let dir = Dir::create_temp_dir("testing").await.unwrap();
            let layout = StorageLayout::new(dir);

            // create a private key file
            let private_key_file = layout.auth_dir().private_key_file();
            private_key_file.write_string("test", false, false).await.unwrap();

            // create the agent file
            let agent_file = layout.agent_file();
            let agent = Agent {
                client_id: "cli_123".to_string(),
            };
            agent_file.write_json(&agent, false, false).await.unwrap();

            let (state, _) = ServerState::new(layout.clone()).await.unwrap();
            let before_record = Utc::now().timestamp();
            std::thread::sleep(std::time::Duration::from_secs(1));
            state.record_activity();
            assert!(state.last_activity.load(Ordering::Relaxed) > before_record as u64);
        }
    }

}
