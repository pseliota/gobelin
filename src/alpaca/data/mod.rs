//! Blocking HTTP access to Alpaca's Market Data API.

pub mod models;

mod pages;
mod retry;
mod stocks;

use std::{thread, time::SystemTime};

use reqwest::{StatusCode, Url, blocking::RequestBuilder};
use serde::de::DeserializeOwned;

use self::models::Feed;
use self::retry::Failure;
pub use self::retry::RetryPolicy;
use super::ClientError;
use super::transport::{self, Transport};

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
    transport: Transport,
    config: DataClientConfig,
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
        let transport = Transport::new(key, secret, base_url)?;
        Ok(Self { transport, config })
    }

    /// Sends the request until it succeeds or the retry policy gives up,
    /// then decodes the JSON body of the 200 response.
    fn fetch<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, ClientError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let attempt = request.try_clone().ok_or(ClientError::InvalidRequest)?;
            let (error, failure) = match self.attempt(attempt) {
                Err(failed) => failed,
                Ok(value) => return Ok(value),
            };
            match failure.and_then(|failure| self.config.retry.delay(attempts, failure)) {
                Some(delay) => thread::sleep(delay),
                None => return Err(error),
            }
        }
    }

    /// Makes one attempt. A failure the retry policy may judge comes with its description.
    fn attempt<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, (ClientError, Option<Failure>)> {
        let response = self.transport.execute(request).map_err(classify)?;
        if response.status() == StatusCode::OK {
            return transport::decode(response).map_err(classify);
        }
        let status = response.status().as_u16();
        let reset_after = response
            .headers()
            .get("x-ratelimit-reset")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| retry::reset_after(value, SystemTime::now()));
        let failure = Failure::Status {
            status,
            reset_after,
        };
        Err((ClientError::HttpStatus { status }, Some(failure)))
    }
}

fn classify(error: ClientError) -> (ClientError, Option<Failure>) {
    let failure = matches!(error, ClientError::Transport { .. }).then_some(Failure::Transport);
    (error, failure)
}

#[cfg(test)]
mod tests;
