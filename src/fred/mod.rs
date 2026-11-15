//! Blocking HTTP access to FRED API version 1 of the Federal Reserve Bank of St. Louis.
//!
//! This product uses the FRED® API but is not endorsed or certified by the Federal Reserve Bank of St. Louis.
//!
//! Series whose notes contain "Copyright" belong to third parties.
//!
//! The API key is sent as the `api_key` query parameter over HTTPS, so request URLs
//! must not be logged.

pub mod models;

mod series;

use reqwest::{Url, header::HeaderMap};
use serde::{Serialize, de::DeserializeOwned};

use crate::transport::{ClientError, RetryPolicy, RetryingTransport, Transport};

const FRED_URL: &str = "https://api.stlouisfed.org/";

#[derive(Debug, Clone, Default)]
pub struct FredConfig {
    pub retry: RetryPolicy,
}

/// A reusable FRED client with a 20-second timeout per attempt.
///
/// Failed attempts are retried as configured by [`RetryPolicy`], sleeping on the
/// calling thread. A call can block for up to `max_attempts` timeouts plus
/// `max_attempts - 1` waits of `max_delay`: 120 seconds with the default policy.
/// The API key and response bodies are not logged.
pub struct FredClient {
    transport: RetryingTransport,
    api_key: String,
}

impl FredClient {
    /// Fails with [`ClientError::InvalidCredentials`] when `api_key` is blank.
    pub fn new(api_key: &str, config: FredConfig) -> Result<Self, ClientError> {
        let base_url = Url::parse(FRED_URL).expect("valid FRED URL");
        Self::build(api_key, base_url, config)
    }

    fn build(api_key: &str, base_url: Url, config: FredConfig) -> Result<Self, ClientError> {
        if api_key.trim().is_empty() {
            return Err(ClientError::InvalidCredentials);
        }
        let transport = Transport::new(base_url, HeaderMap::new())?;
        Ok(Self {
            transport: RetryingTransport::new(transport, config.retry, None),
            api_key: api_key.to_owned(),
        })
    }

    /// Sends a GET with `query`, the API key and the JSON format, and decodes the response.
    fn fetch<T, Q>(&self, segments: &[&str], query: &Q) -> Result<T, ClientError>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let request = self
            .transport
            .get(segments)?
            .query(query)
            .query(&[("api_key", self.api_key.as_str()), ("file_type", "json")]);
        self.transport.fetch(request)
    }
}

#[cfg(test)]
mod tests;
