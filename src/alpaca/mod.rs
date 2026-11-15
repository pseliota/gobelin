pub mod data;
pub mod trading;

use crate::transport::{ClientError, Headers};

fn credential_headers(key: &str, secret: &str) -> Result<Headers, ClientError> {
    let mut headers = Headers::default();
    headers.insert_sensitive("APCA-API-KEY-ID", credential(key)?);
    headers.insert_sensitive("APCA-API-SECRET-KEY", credential(secret)?);
    Ok(headers)
}

fn credential(value: &str) -> Result<&str, ClientError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        return Err(ClientError::InvalidCredentials);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_headers_redacts_values_when_formatted_for_debug() {
        let headers = credential_headers("test-key", "test-secret").unwrap();

        let debug = format!("{headers:?}");

        assert!(headers.iter().all(|header| header.is_sensitive()));
        assert!(!debug.contains("test-key"));
        assert!(!debug.contains("test-secret"));
    }

    #[test]
    fn credential_accepts_value_when_value_is_printable() {
        let value = credential("PK0123456789abcdef");

        assert_eq!(value, Ok("PK0123456789abcdef"));
    }

    #[test]
    fn credential_rejects_value_when_value_is_blank_or_has_control_characters() {
        let cases = [
            "",
            " ",
            "secret\nvalue",
            "secret\rvalue",
            "secret\0",
            "se\tcret",
        ];

        for value in cases {
            let result = credential(value);

            assert_eq!(result, Err(ClientError::InvalidCredentials), "{value:?}");
        }
    }
}
