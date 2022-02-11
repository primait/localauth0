use openssl::error::ErrorStack;
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Provided kid '{0}' is none of in well known jwks kid")]
    UnknownKID(String),

    #[error("Empty jwks store. This should never happen..")]
    EmptyJwks,

    #[error("Provided JWT does not contain a KID")]
    JwtMissingKid,

    #[error(transparent)]
    JWTError(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    #[error(transparent)]
    OpenSSLError(#[from] ErrorStack),

    #[error(transparent)]
    OpenSSLParseUtf8Error(#[from] FromUtf8Error),
}
