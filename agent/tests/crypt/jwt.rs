    // internal crates
    use config_agent::crypt::base64;
    use config_agent::crypt::errors::CryptErr;
    use config_agent::crypt::jwt;
    use config_agent::crypt::jwt::Claims;
    // external crates
    use serde_json::json;
    #[allow(unused_imports)]
    use tracing::{debug, error, info, trace, warn};

    pub mod decode {
        use super::*;

        #[test]
        fn invalid_jwt_format() {
            let token_2_parts = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJkZXZpY2UiLCJleHAiOjE3MjE1MTcwMzQsImlhdCI6MTcyMTQ5NTQzNCwiaXNzIjoiTWlydSIsInN1YiI6Ijc1ODk5YWE0LWIwOGEtNDA0Ny04NTI2LThmMGIxYjgzMjk3MyJ9";
            let result = jwt::decode(token_2_parts);
            assert!(result.is_err());
            assert!(matches!(result, Err(CryptErr::InvalidJWTErr { .. })));

            let token_4_parts = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJkZXZpY2UiLCJleHAiOjE3MjE1MTcwMzQsImlhdCI6MTcyMTQ5NTQzNCwiaXNzIjoiTWlydSIsInN1YiI6Ijc1ODk5YWE0LWIwOGEtNDA0Ny04NTI2LThmMGIxYjgzMjk3MyJ9.UIqAz_V-ZuZLIHUXwLHw-A2CrXBQrpXnJAMlVfmMXYY.arglebargle";
            let result = jwt::decode(token_4_parts);
            assert!(result.is_err());
            assert!(matches!(result, Err(CryptErr::InvalidJWTErr { .. })));
        }

        #[test]
        fn payload_not_decodable() {
            let payload = "arglechargle";
            let token = format!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.UIqAz_V-ZuZLIHUXwLHw-A2CrXBQrpXnJAMlVfmMXYY",
            payload
        );
            let result = jwt::decode(&token);
            println!("Result: {:?}", result);
            assert!(result.is_err());
            assert!(matches!(
                result,
                Err(CryptErr::ConvertBytesToStringErr { .. })
            ));
        }

        #[test]
        fn invalid_payload_format() {
            // missing the issuer
            let invalid_payloads = vec![
                json!({
                    // missing the issuer
                    "aud": "client",
                    "exp": 1721517034,
                    "iat": 1721495434,
                    "sub": "75899aa4-b08a-4047-8526-880b1b832973"
                })
                .to_string(),
                json!({
                    // missing the audience
                    "iss": "miru",
                    "exp": 1721517034,
                    "iat": 1721495434,
                    "sub": "75899aa4-b08a-4047-8526-880b1b832973"
                })
                .to_string(),
                json!({
                    // missing the subject
                    "iss": "miru",
                    "aud": "client",
                    "exp": 1721517034,
                    "iat": 1721495434,
                })
                .to_string(),
                json!({
                    // missing the expiration time
                    "iss": "miru",
                    "aud": "client",
                    "iat": 1721495434,
                    "sub": "75899aa4-b08a-4047-8526-880b1b832973"
                })
                .to_string(),
                json!({
                    // missing the issued at time
                    "iss": "miru",
                    "aud": "client",
                    "exp": 1721517034,
                    "sub": "75899aa4-b08a-4047-8526-880b1b832973"
                })
                .to_string(),
            ];

            for payload in invalid_payloads {
                let token = format!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.UIqAz_V-ZuZLIHUXwLHw-A2CrXBQrpXnJAMlVfmMXYY",
            base64::encode_string_url_safe_no_pad(&payload)
            );
                let result = jwt::decode(&token);
                assert!(result.is_err());
                assert!(matches!(result, Err(CryptErr::InvalidJWTPayloadErr { .. })));
            }
        }

        #[test]
        fn success() {
            let payload = json!({
                "iss": "miru",
                "aud": "client",
                "exp": 1721517034,
                "iat": 1721495434,
                "sub": "75899aa4-b08a-4047-8526-880b1b832973"
            })
            .to_string();

            let token = format!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.UIqAz_V-ZuZLIHUXwLHw-A2CrXBQrpXnJAMlVfmMXYY",
            base64::encode_string_url_safe_no_pad(&payload)
        );
            let claims = jwt::decode(&token).unwrap();
            let expected = Claims {
                iss: "miru".to_string(),
                aud: "client".to_string(),
                exp: 1721517034,
                iat: 1721495434,
                sub: "75899aa4-b08a-4047-8526-880b1b832973".to_string(),
            };
            assert_eq!(claims, expected);
        }
    }

    pub mod extract_client_id {
        use super::*;

        #[test]
        fn payload_not_decodable() {
            let payload = "arglechargle";
            let token = format!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.UIqAz_V-ZuZLIHUXwLHw-A2CrXBQrpXnJAMlVfmMXYY",
            payload
        );
            let result = jwt::extract_client_id(&token).unwrap_err();
            println!("Result: {:?}", result);
            assert!(matches!(result, CryptErr::ConvertBytesToStringErr { .. }));
        }

        #[test]
        fn success() {
            let payload = json!({
                "iss": "miru",
                "aud": "client",
                "exp": 1721517034,
                "iat": 1721495434,
                "sub": "75899aa4-b08a-4047-8526-880b1b832973"
            })
            .to_string();

            let token = format!(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.UIqAz_V-ZuZLIHUXwLHw-A2CrXBQrpXnJAMlVfmMXYY",
            base64::encode_string_url_safe_no_pad(&payload)
        );
            let client_id = jwt::extract_client_id(&token).unwrap();
            assert_eq!(client_id, "75899aa4-b08a-4047-8526-880b1b832973");
        }
    }

    pub mod validate_claims {
        use super::*;

        #[test]
        fn client_claims_invalid() {
            let now = chrono::Utc::now().timestamp();
            let invalid_claims = vec![
                // issuer isn't miru
                Claims {
                    iss: "Uncle Sam".to_string(),
                    aud: "client".to_string(),
                    iat: now,
                    exp: now + 1000,
                    sub: "75899aa4-b08a-4047-8526-880b1b832973".to_string(),
                },
                // audience isn't device
                Claims {
                    iss: "miru".to_string(),
                    aud: "user".to_string(),
                    iat: now,
                    exp: now + 1000,
                    sub: "75899aa4-b08a-4047-8526-880b1b832973".to_string(),
                },
                // issued at time is in the future
                Claims {
                    iss: "miru".to_string(),
                    aud: "client".to_string(),
                    iat: now + 1000,
                    exp: now + 1000,
                    sub: "75899aa4-b08a-4047-8526-880b1b832973".to_string(),
                },
                // expiration time is in the past
                Claims {
                    iss: "miru".to_string(),
                    aud: "client".to_string(),
                    iat: now,
                    exp: now - 1,
                    sub: "75899aa4-b08a-4047-8526-880b1b832973".to_string(),
                },
            ];
            for claim in invalid_claims {
                let result = jwt::validate_claims(claim);
                assert!(result.is_err());
                assert!(matches!(result, Err(CryptErr::InvalidJWTErr { .. })));
            }
        }

        #[test]
        fn client_claims_valid() {
            let now = chrono::Utc::now().timestamp();
            let claim = Claims {
                iss: "miru".to_string(),
                aud: "client".to_string(),
                iat: now,
                exp: now + 1000,
                sub: "75899aa4-b08a-4047-8526-880b1b832973".to_string(),
            };
            let result = jwt::validate_claims(claim);
            assert!(result.is_ok());
        }
    }

    #[test]
    #[ignore]
    fn test_validate() {
        // testing this would be redundant since it's such a simple wrapper
    }
