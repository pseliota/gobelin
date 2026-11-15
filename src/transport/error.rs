use std::{error::Error, fmt};

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
                f.write_str("Credentials are empty or contain invalid characters")
            }
            Self::Initialization => f.write_str("Could not initialize the HTTP client"),
            Self::InvalidPath => f.write_str("An endpoint identifier is empty or invalid"),
            Self::InvalidRequest => f.write_str("Could not construct the HTTP request"),
            Self::Transport { timed_out: true } => {
                f.write_str("The request timed out; its outcome may be unknown")
            }
            Self::Transport { timed_out: false } => {
                f.write_str("The request failed during transport; its outcome may be unknown")
            }
            Self::HttpStatus { status } => {
                write!(f, "The server returned unexpected HTTP status {status}")
            }
            Self::InvalidResponse => {
                f.write_str("The server returned an unexpected response format")
            }
        }
    }
}

impl Error for ClientError {}
