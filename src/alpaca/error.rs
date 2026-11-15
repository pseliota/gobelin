use std::{error::Error, fmt};

/// Sanitized failures; response bodies and credential values are never included.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClientError {
    InvalidCredentials,
    Initialization,
    InvalidPath,
    InvalidRequest,
    Transport { timed_out: bool },
    HttpStatus { status: u16 },
    InvalidResponse,
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCredentials => {
                f.write_str("Credentials are empty or contain invalid header characters")
            }
            Self::Initialization => f.write_str("Could not initialize the HTTP client"),
            Self::InvalidPath => f.write_str("An endpoint identifier is empty or invalid"),
            Self::InvalidRequest => f.write_str("Could not construct the HTTP request"),
            Self::Transport { timed_out: true } => {
                f.write_str("Alpaca request timed out; its outcome may be unknown")
            }
            Self::Transport { timed_out: false } => {
                f.write_str("Alpaca request failed during transport; its outcome may be unknown")
            }
            Self::HttpStatus { status } => {
                write!(f, "Alpaca returned unexpected HTTP status {status}")
            }
            Self::InvalidResponse => f.write_str("Alpaca returned an unexpected response format"),
        }
    }
}

impl Error for ClientError {}
