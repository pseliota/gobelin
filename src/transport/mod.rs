//! Blocking HTTP plumbing shared by the API clients.

mod error;
mod retry;
#[cfg(test)]
pub(crate) mod test_server;

use std::{
    thread,
    time::{Duration, SystemTime},
};

use reqwest::{
    Method, StatusCode, Url,
    blocking::{Client, RequestBuilder, Response},
    header::HeaderMap,
    redirect::Policy,
};
use serde::{Serialize, de::DeserializeOwned};

pub use self::error::ClientError;
use self::retry::Failure;
pub use self::retry::RetryPolicy;

/// Reads the time until the rate limit resets from response headers, given the current time.
pub(crate) type RateLimitHint = fn(&HeaderMap, SystemTime) -> Option<Duration>;

/// A connection to one API base URL with a 20-second request timeout.
/// Every request is sent once: redirects and retries are disabled.
pub(crate) struct Transport {
    http: Client,
    base_url: Url,
}

impl Transport {
    /// Sends `default_headers` with every request.
    pub(crate) fn new(base_url: Url, default_headers: HeaderMap) -> Result<Self, ClientError> {
        let mut builder = Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(20))
            .redirect(Policy::none())
            .retry(reqwest::retry::never());
        // Local HTTP tests must not use system proxy settings.
        if base_url.scheme() == "http" {
            builder = builder.no_proxy();
        }
        let http = builder.build().map_err(|_| ClientError::Initialization)?;
        Ok(Self { http, base_url })
    }

    #[cfg(test)]
    pub(crate) fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Starts a request to the base URL extended by the percent-encoded segments.
    pub(crate) fn request(
        &self,
        method: Method,
        segments: &[&str],
    ) -> Result<RequestBuilder, ClientError> {
        if segments
            .iter()
            .any(|s| s.is_empty() || *s == "." || *s == "..")
        {
            return Err(ClientError::InvalidPath);
        }
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| ClientError::InvalidPath)?
            .pop_if_empty()
            .extend(segments);
        Ok(self.http.request(method, url))
    }

    /// Sends the request and decodes the JSON body of a 200 response.
    pub(crate) fn json<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, ClientError> {
        decode(self.send(request, StatusCode::OK)?)
    }

    /// Sends the request and fails unless the response has the expected status.
    pub(crate) fn send(
        &self,
        request: RequestBuilder,
        expected: StatusCode,
    ) -> Result<Response, ClientError> {
        let response = execute(request)?;
        if response.status() != expected {
            return Err(ClientError::HttpStatus {
                status: response.status().as_u16(),
            });
        }
        Ok(response)
    }
}

/// A GET request; only [`RetryingTransport`] creates one.
pub(crate) struct GetRequest(RequestBuilder);

impl GetRequest {
    /// Appends the serialized pairs to the query string.
    pub(crate) fn query<Q: Serialize + ?Sized>(self, query: &Q) -> Self {
        Self(self.0.query(query))
    }
}

/// A [`Transport`] that only sends GET requests and retries them as `policy` allows,
/// sleeping on the calling thread.
pub(crate) struct RetryingTransport {
    transport: Transport,
    policy: RetryPolicy,
    rate_limit: Option<RateLimitHint>,
}

impl RetryingTransport {
    /// Without a `rate_limit` hint, a 429 is retried with exponential backoff.
    pub(crate) fn new(
        transport: Transport,
        policy: RetryPolicy,
        rate_limit: Option<RateLimitHint>,
    ) -> Self {
        Self {
            transport,
            policy,
            rate_limit,
        }
    }

    #[cfg(test)]
    pub(crate) fn base_url(&self) -> &Url {
        self.transport.base_url()
    }

    /// Starts a GET to the base URL extended by the percent-encoded segments.
    pub(crate) fn get(&self, segments: &[&str]) -> Result<GetRequest, ClientError> {
        self.transport
            .request(Method::GET, segments)
            .map(GetRequest)
    }

    /// Sends the request until it succeeds or the retry policy gives up,
    /// then decodes the JSON body of the 200 response.
    /// Returns the last attempt's error when it gives up.
    pub(crate) fn fetch<T: DeserializeOwned>(&self, request: GetRequest) -> Result<T, ClientError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let attempt = request.0.try_clone().ok_or(ClientError::InvalidRequest)?;
            let (error, failure) = match self.attempt(attempt) {
                Err(failed) => failed,
                Ok(value) => return Ok(value),
            };
            match failure.and_then(|failure| self.policy.delay(attempts, failure)) {
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
        let response = execute(request).map_err(classify)?;
        if response.status() == StatusCode::OK {
            return decode(response).map_err(classify);
        }
        let status = response.status().as_u16();
        let reset_after = self
            .rate_limit
            .and_then(|hint| hint(response.headers(), SystemTime::now()));
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

/// Sends the request and returns the response whatever its status.
fn execute(request: RequestBuilder) -> Result<Response, ClientError> {
    request.send().map_err(|error| {
        if error.is_builder() {
            ClientError::InvalidRequest
        } else {
            ClientError::Transport {
                timed_out: error.is_timeout(),
            }
        }
    })
}

pub(crate) fn decode<T: DeserializeOwned>(response: Response) -> Result<T, ClientError> {
    response.json().map_err(|error| {
        if error.is_timeout() || error.is_body() {
            ClientError::Transport {
                timed_out: error.is_timeout(),
            }
        } else {
            ClientError::InvalidResponse
        }
    })
}

#[cfg(test)]
mod tests;
