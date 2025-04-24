// internal crates
use crate::auth::errors::{
    AuthErr,
    AuthCryptErr,
    SerdeErr,
};
use crate::crypt::{
    base64,
    rsa,
};
use crate::filesys::file::File;
use crate::trace;

// external crates
use chrono::{Utc, Duration};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct IssueTokenClaim {
    pub client_id: String,
    pub nonce: String,
    pub expiration: i64,
}

pub struct IssueTokenRequest {
    pub claims: IssueTokenClaim,
    pub signature: String,
}

pub async fn prepare_issue_token_request(
    client_id: &str,
    private_key_file: &File,
) -> Result<IssueTokenRequest, AuthErr> {
    // prepare the claims
    let nonce = Uuid::new_v4().to_string();
    let expiration = Utc::now() + Duration::minutes(2);
    let claims = IssueTokenClaim {
        client_id: client_id.to_string(),
        nonce,
        expiration: expiration.timestamp(),
    };

    // serialize the claims into a JSON byte vector
    let claims_bytes = serde_json::to_vec(&claims).map_err(|e| AuthErr::SerdeErr(SerdeErr {
        source: e,
        trace: trace!(),
    }))?;

    // sign the claims
    let signature_bytes = rsa::sign(private_key_file, &claims_bytes).await.map_err(|e| AuthErr::CryptErr(AuthCryptErr{
        source: e,
        trace: trace!(),
    }))?;
    let signature = base64::encode_bytes_standard(&signature_bytes);

    Ok(IssueTokenRequest {
        claims,
        signature,
    })
}