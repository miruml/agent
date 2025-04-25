#[cfg(test)]
mod tests {

    pub mod run {
        use super::*;

        #[tokio::test]
        async fn invalid_server_state_initialization() {
            assert!(false);
        }

        #[tokio::test]
        async fn max_runtime_reached() {
            assert!(false);
        }

        #[tokio::test]
        async fn idle_timeout_reached() {
            assert!(false);
        }

        #[tokio::test]
        async fn ctrl_c_received() {
            assert!(false);
        }
    }

}