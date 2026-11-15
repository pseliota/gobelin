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

pub struct FredClient {
    transport: RetryingTransport,
    api_key: String,
}

impl FredClient {
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
