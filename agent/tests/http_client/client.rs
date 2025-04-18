#[cfg(test)]
mod tests {
    // internal crates
    use std::time::{Duration, Instant};

    // internal crates
    use config_agent::errors::MiruError;
    use config_agent::http_client::client::HTTPClient;
    use config_agent::http_client::errors::HTTPErr;

    // external crates
    use moka::future::Cache;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

pub mod build_get_request {
    use super::*;

    #[tokio::test]
    async fn get_example_dot_com() {
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "https://example.com/",
            None,
        ).unwrap();
        let result = http_client.send(request, Duration::from_secs(1)).await.unwrap();
        assert!(result.status().is_success());
    }

    #[tokio::test]
    async fn post_to_httpbin() {
        let http_client = HTTPClient::new().await;
        
        // Create a simple JSON payload
        let payload = serde_json::json!({
            "test": "data",
            "number": 42
        });
        
        let body = serde_json::to_string(&payload).unwrap();
        let request = http_client.build_post_request(
            "https://httpbin.org/post",  // httpbin.org is commonly used for testing HTTP requests
            body,
            None,
        ).unwrap();
        
        let response = http_client.send(request, Duration::from_secs(5)).await.unwrap();
        assert!(response.status().is_success());
        
        // Parse and verify the response
        let text = response.text().await.unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        
        // httpbin.org echoes back the JSON data in the "json" field
        assert_eq!(json["json"]["test"], "data");
        assert_eq!(json["json"]["number"], 42);
    }
}

pub mod send {
    use super::*;

pub mod success {
    use super::*;

    #[tokio::test]
    async fn get_example_dot_com() {
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "https://example.com/",
            None,
        ).unwrap();
        let result = http_client.send(request, Duration::from_secs(1)).await.unwrap();
        assert!(result.status().is_success());
    }
}

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn network_connection_error() {
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "http://localhost:8080",
            None,
        ).unwrap();
        let result = http_client.send(request, Duration::from_secs(1)).await.unwrap_err();
        assert!(result.is_network_connection_error());
    }

    #[tokio::test]
    async fn timeout_error() {
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "https://example.com/",
            None,
        ).unwrap();
        let result = http_client.send(request, Duration::from_millis(1)).await.unwrap_err();
        assert!(matches!(result, HTTPErr::TimeoutErr { .. }));
    }
}
}

pub mod send_cached {
    use super::*;

pub mod success {
    use super::*;

    #[tokio::test]
    async fn sequential_cache_hit() {
        let http_client = HTTPClient::new().await;
        let url = "https://example.com/";

        // send the first request
        let start = Instant::now();
        let request = http_client.build_get_request(
            url,
            None,
        ).unwrap();
        http_client.send_cached(
            url.to_string(),
            request,
            Duration::from_secs(1),
        ).await.unwrap();
        let duration = start.elapsed();
        println!("duration {}", duration.as_millis());
        assert!(duration > Duration::from_millis(10));

        // send subsequent requests and check they are cached
        for _ in 0..5 {
            let start = Instant::now();
            let request = http_client.build_get_request(
                url,
                None,
            ).unwrap();
            http_client.send_cached(
                url.to_string(),
                request,
                Duration::from_secs(1),
            ).await.unwrap();
            let duration = start.elapsed();
            println!("duration {}", duration.as_millis());
            assert!(duration < Duration::from_millis(300));
        }
    }

    #[tokio::test]
    async fn errors_not_cached() {
        let http_client = HTTPClient::new().await;
        let url = "https://httpstat.us/404";

        // send the first request
        let start = Instant::now();
        let request = http_client.build_get_request(
            url,
            None,
        ).unwrap();
        http_client.send_cached(
            url.to_string(),
            request,
            Duration::from_secs(1),
        ).await.unwrap_err();
        let duration = start.elapsed();
        println!("duration {}", duration.as_millis());
        assert!(duration > Duration::from_millis(10));

        // send subsequent requests and check they are not cached
        for _ in 0..5 {
            let start = Instant::now();
            let request = http_client.build_get_request(
                url,
                None,
            ).unwrap();
            http_client.send_cached(
                url.to_string(),
                request,
                Duration::from_secs(1),
            ).await.unwrap_err();
                let duration = start.elapsed();
                println!("duration {}", duration.as_millis());
                assert!(duration > Duration::from_millis(10));
        }
    }

    #[tokio::test]
    async fn cache_expired() {
        let url = "https://example.com/";
        let http_client = HTTPClient::new_with(
            url,
            Duration::from_secs(1),
            Cache::builder()
                .time_to_live(Duration::from_millis(100))
                .build(),
        );

        // send the first request
        let start = Instant::now();
        let request = http_client.build_get_request(
            url,
            None,
        ).unwrap();
        http_client.send_cached(
            url.to_string(),
            request,
            Duration::from_secs(1),
        ).await.unwrap();
        let duration = start.elapsed();
        println!("duration {}", duration.as_millis());
        assert!(duration > Duration::from_millis(10));

        // wait for the cache to expire
        std::thread::sleep(Duration::from_secs(1));

        // send subsequent requests and check they are not cached
        let start = Instant::now();
        let request = http_client.build_get_request(
            url,
            None,
        ).unwrap();
        http_client.send_cached(
            url.to_string(),
            request,
            Duration::from_secs(1),
        ).await.unwrap();
            let duration = start.elapsed();
            println!("duration {}", duration.as_millis());
            assert!(duration > Duration::from_millis(10));
    }
}

pub mod errors {
    use super::*;

    #[tokio::test]
    async fn network_connection_error() {
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "http://localhost:8080",
            None,
        ).unwrap();
        let result = http_client.send_cached(
            "test".to_string(),
            request,
            Duration::from_secs(1),
        ).await.unwrap_err();
        assert!(result.is_network_connection_error());
    }

    #[tokio::test]
    async fn timeout_error() {
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "https://example.com/",
            None,
        ).unwrap();
        let result = http_client.send_cached(
            "test".to_string(),
            request,
            Duration::from_millis(1),
        ).await.unwrap_err();
        assert!(matches!(result, HTTPErr::TimeoutErr { .. }));
    }
}
}

pub mod handle_response {
    use super::*;

    #[tokio::test]
    async fn endpoint_not_found() {
        // make a request to a non-existent endpoint
        let http_client = HTTPClient::new().await;
        let request = http_client.build_get_request(
            "https://httpstat.us/404",
            None,
        ).unwrap();
        let resp = http_client.send(request, Duration::from_secs(1)).await.unwrap();

        // call the handle_response method
        let response = http_client.handle_response(resp).await.unwrap_err();
        assert!(matches!(response, HTTPErr::ResponseFailed { .. }));
    }
}
}