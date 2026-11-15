pub mod models;

mod series;

use serde::{Serialize, de::DeserializeOwned};
use url::Url;

use crate::transport::{
    ClientError, Headers, HttpExecutor, RetryPolicy, RetryingTransport, Transport, TransportConfig,
};

const FRED_URL: &str = "https://api.stlouisfed.org/";

#[derive(Debug, Clone, Default)]
pub struct FredConfig {
    pub retry: RetryPolicy,
    pub transport: TransportConfig,
}

pub struct FredClient {
    transport: RetryingTransport,
    api_key: String,
}

impl FredClient {
    pub fn new(api_key: &str, config: FredConfig) -> Result<Self, ClientError> {
        let executor = config.transport.executor()?;
        Self::with_executor(api_key, config, executor)
    }

    fn with_executor(
        api_key: &str,
        config: FredConfig,
        executor: Box<dyn HttpExecutor>,
    ) -> Result<Self, ClientError> {
        if api_key.trim().is_empty() {
            return Err(ClientError::InvalidCredentials);
        }
        let base_url = Url::parse(FRED_URL).expect("valid FRED URL");
        let transport = Transport::new(base_url, Headers::default(), executor);
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
            .query(query)?
            .query(&[("api_key", self.api_key.as_str()), ("file_type", "json")])?;
        self.transport.fetch(request)
    }
}

#[cfg(test)]
mod tests;
