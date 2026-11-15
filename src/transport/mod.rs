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

pub(crate) type RateLimitHint = fn(&HeaderMap, SystemTime) -> Option<Duration>;

pub(crate) struct Transport {
    http: Client,
    base_url: Url,
}

impl Transport {
    pub(crate) fn new(base_url: Url, default_headers: HeaderMap) -> Result<Self, ClientError> {
        let mut builder = Client::builder()
            .default_headers(default_headers)
            .timeout(Duration::from_secs(20))
            .redirect(Policy::none())
            .retry(reqwest::retry::never());
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

    pub(crate) fn json<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, ClientError> {
        decode(self.send(request, StatusCode::OK)?)
    }

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

pub(crate) struct GetRequest(RequestBuilder);

impl GetRequest {
    pub(crate) fn query<Q: Serialize + ?Sized>(self, query: &Q) -> Self {
        Self(self.0.query(query))
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
        self.transport
            .request(Method::GET, segments)
            .map(GetRequest)
    }

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
