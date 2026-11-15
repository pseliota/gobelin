//! Blocking HTTP plumbing shared by the Alpaca clients.

use std::time::Duration;

use reqwest::{
    Method, StatusCode, Url,
    blocking::{Client, RequestBuilder, Response},
    header::{HeaderMap, HeaderValue},
    redirect::Policy,
};
use serde::de::DeserializeOwned;

use super::error::ClientError;

/// An authenticated connection to one API base URL with a 20-second request timeout.
/// Redirects and retries are disabled.
pub(super) struct Transport {
    http: Client,
    base_url: Url,
}

impl Transport {
    pub(super) fn new(key: &str, secret: &str, base_url: Url) -> Result<Self, ClientError> {
        let mut headers = HeaderMap::new();
        headers.insert("APCA-API-KEY-ID", credential_header(key)?);
        headers.insert("APCA-API-SECRET-KEY", credential_header(secret)?);
        let mut builder = Client::builder()
            .default_headers(headers)
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
    pub(super) fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Starts a request to the base URL extended by the percent-encoded segments.
    pub(super) fn request(
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
    pub(super) fn json<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, ClientError> {
        decode(self.send(request, StatusCode::OK)?)
    }

    /// Sends the request and fails unless the response has the expected status.
    pub(super) fn send(
        &self,
        request: RequestBuilder,
        expected: StatusCode,
    ) -> Result<Response, ClientError> {
        let response = request.send().map_err(|error| {
            if error.is_builder() {
                ClientError::InvalidRequest
            } else {
                ClientError::Transport {
                    timed_out: error.is_timeout(),
                }
            }
        })?;
        if response.status() != expected {
            return Err(ClientError::HttpStatus {
                status: response.status().as_u16(),
            });
        }
        Ok(response)
    }
}

pub(super) fn decode<T: DeserializeOwned>(response: Response) -> Result<T, ClientError> {
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

fn credential_header(value: &str) -> Result<HeaderValue, ClientError> {
    if value.trim().is_empty() {
        return Err(ClientError::InvalidCredentials);
    }
    let mut header = HeaderValue::from_str(value).map_err(|_| ClientError::InvalidCredentials)?;
    header.set_sensitive(true);
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_header_redacts_value_when_formatted_for_debug() {
        let header = credential_header("test-secret").unwrap();

        let debug = format!("{header:?}");

        assert!(header.is_sensitive());
        assert!(!debug.contains("test-secret"));
    }
}
