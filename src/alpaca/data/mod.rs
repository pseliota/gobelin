//! Blocking HTTP access to Alpaca's Market Data API.

pub mod models;

mod pages;
mod rate_limit;
mod stocks;

use reqwest::Url;

use self::models::Feed;
use super::credential_headers;
use crate::transport::{ClientError, RetryPolicy, RetryingTransport, Transport};

const DATA_URL: &str = "https://data.alpaca.markets/";

#[derive(Debug, Clone, Default)]
pub struct DataClientConfig {
    /// Feed sent with every stock request. `None` leaves the choice to Alpaca.
    pub feed: Option<Feed>,
    pub retry: RetryPolicy,
}

/// A reusable market data client with a 20-second timeout per attempt.
///
/// Failed attempts are retried as configured by [`RetryPolicy`], sleeping on the
/// calling thread. A call can block for up to `max_attempts` timeouts plus
/// `max_attempts - 1` waits of `max_delay`: 120 seconds with the default policy.
/// Credentials and response bodies are not logged.
pub struct AlpacaDataClient {
    transport: RetryingTransport,
    feed: Option<Feed>,
}

impl AlpacaDataClient {
    pub fn new(key: &str, secret: &str, config: DataClientConfig) -> Result<Self, ClientError> {
        let base_url = Url::parse(DATA_URL).expect("valid data URL");
        Self::build(key, secret, base_url, config)
    }

    fn build(
        key: &str,
        secret: &str,
        base_url: Url,
        config: DataClientConfig,
    ) -> Result<Self, ClientError> {
        let transport = Transport::new(base_url, credential_headers(key, secret)?)?;
        let transport =
            RetryingTransport::new(transport, config.retry, Some(rate_limit::reset_hint));
        Ok(Self {
            transport,
            feed: config.feed,
        })
    }
}

#[cfg(test)]
mod tests;
