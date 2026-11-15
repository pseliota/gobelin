pub mod models;

mod pages;
mod rate_limit;
mod stocks;

use reqwest::Url;

use self::models::Feed;
use super::credential_headers;
use crate::transport::{ClientError, RetryPolicy, RetryingTransport, Transport, TransportConfig};

const DATA_URL: &str = "https://data.alpaca.markets/";

#[derive(Debug, Clone, Default)]
pub struct DataClientConfig {
    pub feed: Option<Feed>,
    pub retry: RetryPolicy,
    pub transport: TransportConfig,
}

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
        let transport = Transport::new(
            base_url,
            credential_headers(key, secret)?,
            &config.transport,
        )?;
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
