use reqwest::{
    Method,
    blocking::Client,
    header::{HeaderMap, HeaderName, HeaderValue},
    redirect::Policy,
};

use super::{
    ClientError, TransportConfig,
    http::{Headers, HttpExecutor, HttpMethod, HttpRequest, HttpResponse, TransportFailure},
};

pub(super) struct ReqwestExecutor {
    client: Client,
}

impl ReqwestExecutor {
    pub(super) fn new(config: &TransportConfig) -> Result<Self, ClientError> {
        Self::build(config, true)
    }

    #[cfg(test)]
    pub(super) fn without_proxy(config: &TransportConfig) -> Result<Self, ClientError> {
        Self::build(config, false)
    }

    fn build(config: &TransportConfig, system_proxy: bool) -> Result<Self, ClientError> {
        let mut builder = Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .user_agent(config.user_agent.as_str())
            .redirect(Policy::none())
            .retry(reqwest::retry::never());
        if !system_proxy {
            builder = builder.no_proxy();
        }
        let client = builder.build().map_err(|_| ClientError::Initialization)?;
        Ok(Self { client })
    }
}

impl HttpExecutor for ReqwestExecutor {
    fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportFailure> {
        let mut builder = self
            .client
            .request(method(request.method), request.url)
            .headers(header_map(&request.headers)?);
        if let Some(body) = request.body {
            builder = builder.body(body);
        }
        let response = builder.send().map_err(failure)?;
        let status = response.status().as_u16();
        let headers = owned_headers(response.headers());
        let body = response.bytes().map_err(failure)?.to_vec();
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

fn method(method: HttpMethod) -> Method {
    match method {
        HttpMethod::Get => Method::GET,
        HttpMethod::Post => Method::POST,
        HttpMethod::Patch => Method::PATCH,
        HttpMethod::Delete => Method::DELETE,
    }
}

fn header_map(headers: &Headers) -> Result<HeaderMap, TransportFailure> {
    let mut map = HeaderMap::new();
    for header in headers.iter() {
        let name = HeaderName::from_bytes(header.name().as_bytes())
            .map_err(|_| TransportFailure::Unsendable)?;
        let mut value =
            HeaderValue::from_str(header.value()).map_err(|_| TransportFailure::Unsendable)?;
        value.set_sensitive(header.is_sensitive());
        map.insert(name, value);
    }
    Ok(map)
}

fn owned_headers(map: &HeaderMap) -> Headers {
    let mut headers = Headers::default();
    for (name, value) in map {
        if let Ok(value) = value.to_str() {
            headers.insert(name.as_str(), value);
        }
    }
    headers
}

fn failure(error: reqwest::Error) -> TransportFailure {
    if error.is_builder() {
        TransportFailure::Unsendable
    } else {
        TransportFailure::Interrupted {
            timed_out: error.is_timeout(),
        }
    }
}

#[cfg(test)]
mod tests;
