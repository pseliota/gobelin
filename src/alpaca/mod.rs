pub mod data;
pub mod trading;

use reqwest::header::{HeaderMap, HeaderValue};

use crate::transport::ClientError;

fn credential_headers(key: &str, secret: &str) -> Result<HeaderMap, ClientError> {
    let mut headers = HeaderMap::new();
    headers.insert("APCA-API-KEY-ID", credential_header(key)?);
    headers.insert("APCA-API-SECRET-KEY", credential_header(secret)?);
    Ok(headers)
}

fn credential_header(value: &str) -> Result<HeaderValue, ClientError> {
    if value.trim().is_empty() {
        return Err(ClientError::InvalidCredentials);
    }
    let mut header = HeaderValue::from_str(value).map_err(|_| ClientError::InvalidCredentials)?;
    header.set_sensitive(true);
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_header_redacts_value_when_formatted_for_debug() {
        let header = credential_header("test-secret").unwrap();

        let debug = format!("{header:?}");

        assert!(header.is_sensitive());
        assert!(!debug.contains("test-secret"));
    }
}
