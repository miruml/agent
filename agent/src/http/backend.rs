pub enum BackendErrorCodes {
    InternalServerError,
    InvalidJWTAuth,
}

impl BackendErrorCodes {
    pub fn as_str(&self) -> &str {
        match self {
            Self::InternalServerError => "internal_server_error",
            Self::InvalidJWTAuth => "invalid_jwt_auth",
        }
    }
}
