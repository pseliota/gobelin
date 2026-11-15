mod config;
mod error;
mod executor;
mod http;
mod request;
mod retry;
#[cfg(test)]
pub(crate) mod testing;

use std::{
    thread,
    time::{Duration, SystemTime},
};

use serde::de::DeserializeOwned;
use url::Url;

pub use self::config::TransportConfig;
pub use self::error::ClientError;
pub(crate) use self::http::{Headers, HttpExecutor, HttpMethod, HttpResponse, status};
use self::http::{HttpRequest, TransportFailure};
pub(crate) use self::request::{GetRequest, Request};
use self::retry::Failure;
pub use self::retry::RetryPolicy;

pub(crate) type RateLimitHint = fn(&Headers, SystemTime) -> Option<Duration>;

pub(crate) struct Transport {
    executor: Box<dyn HttpExecutor>,
    base_url: Url,
    default_headers: Headers,
}

impl Transport {
    pub(crate) fn new(
        base_url: Url,
        default_headers: Headers,
        executor: Box<dyn HttpExecutor>,
    ) -> Self {
        Self {
            executor,
            base_url,
            default_headers,
        }
    }

    #[cfg(test)]
    pub(crate) fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub(crate) fn request(
        &self,
        method: HttpMethod,
        segments: &[&str],
    ) -> Result<Request, ClientError> {
        let url = self.url(segments)?;
        Ok(Request::new(method, url, self.default_headers.clone()))
    }

    pub(crate) fn json<T: DeserializeOwned>(&self, request: Request) -> Result<T, ClientError> {
        decode(&self.send(request, status::OK)?)
    }

    pub(crate) fn send(
        &self,
        request: Request,
        expected: u16,
    ) -> Result<HttpResponse, ClientError> {
        let response = self.execute(request.into_http())?;
        if response.status != expected {
            return Err(ClientError::HttpStatus {
                status: response.status,
            });
        }
        Ok(response)
    }

    fn url(&self, segments: &[&str]) -> Result<Url, ClientError> {
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
        Ok(url)
    }

    fn execute(&self, request: HttpRequest) -> Result<HttpResponse, ClientError> {
        self.executor
            .execute(request)
            .map_err(|failure| match failure {
                TransportFailure::Unsendable => ClientError::InvalidRequest,
                TransportFailure::Interrupted { timed_out } => ClientError::Transport { timed_out },
            })
    }
}

pub(crate) struct RetryingTransport {
    transport: Transport,
    policy: RetryPolicy,
    rate_limit: Option<RateLimitHint>,
}

impl RetryingTransport {
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

    pub(crate) fn get(&self, segments: &[&str]) -> Result<GetRequest, ClientError> {
        let url = self.transport.url(segments)?;
        Ok(GetRequest::new(url, self.transport.default_headers.clone()))
    }

    pub(crate) fn fetch<T: DeserializeOwned>(&self, request: GetRequest) -> Result<T, ClientError> {
        let mut attempts = 0;
        loop {
            attempts += 1;
            let (error, failure) = match self.attempt(request.to_http()) {
                Err(failed) => failed,
                Ok(value) => return Ok(value),
            };
            match failure.and_then(|failure| self.policy.delay(attempts, failure)) {
                Some(delay) => thread::sleep(delay),
                None => return Err(error),
            }
        }
    }

    fn attempt<T: DeserializeOwned>(
        &self,
        request: HttpRequest,
    ) -> Result<T, (ClientError, Option<Failure>)> {
        let response = self.transport.execute(request).map_err(classify)?;
        if response.status == status::OK {
            return decode(&response).map_err(classify);
        }
        let status = response.status;
        let reset_after = self
            .rate_limit
            .and_then(|hint| hint(&response.headers, SystemTime::now()));
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

pub(crate) fn decode<T: DeserializeOwned>(response: &HttpResponse) -> Result<T, ClientError> {
    serde_json::from_slice(&response.body).map_err(|_| ClientError::InvalidResponse)
}

#[cfg(test)]
mod tests;
