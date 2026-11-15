use std::fmt;

use url::Url;

pub(crate) mod status {
    pub(crate) const OK: u16 = 200;
    pub(crate) const NO_CONTENT: u16 = 204;
    pub(crate) const MULTI_STATUS: u16 = 207;
}

pub(crate) trait HttpExecutor: Send + Sync {
    fn execute(&self, request: HttpRequest) -> Result<HttpResponse, TransportFailure>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HttpMethod {
    Get,
    Post,
    Patch,
    Delete,
}

#[derive(Clone)]
pub(crate) struct HttpRequest {
    pub(crate) method: HttpMethod,
    pub(crate) url: Url,
    pub(crate) headers: Headers,
    pub(crate) body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HttpResponse {
    pub(crate) status: u16,
    pub(crate) headers: Headers,
    pub(crate) body: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransportFailure {
    Unsendable,
    Interrupted { timed_out: bool },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Headers(Vec<Header>);

impl Headers {
    pub(crate) fn insert(&mut self, name: &str, value: &str) {
        self.replace(name, value, false);
    }

    pub(crate) fn insert_sensitive(&mut self, name: &str, value: &str) {
        self.replace(name, value, true);
    }

    pub(crate) fn get(&self, name: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_str())
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Header> {
        self.0.iter()
    }

    fn replace(&mut self, name: &str, value: &str, sensitive: bool) {
        let name = name.to_ascii_lowercase();
        self.0.retain(|header| header.name != name);
        self.0.push(Header {
            name,
            value: value.to_owned(),
            sensitive,
        });
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Header {
    name: String,
    value: String,
    sensitive: bool,
}

impl Header {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn value(&self) -> &str {
        &self.value
    }

    pub(crate) fn is_sensitive(&self) -> bool {
        self.sensitive
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = if self.sensitive {
            "<redacted>"
        } else {
            self.value.as_str()
        };
        f.debug_struct("Header")
            .field("name", &self.name)
            .field("value", &value)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_ignores_case_when_name_differs_in_case() {
        let mut headers = Headers::default();
        headers.insert("X-RateLimit-Reset", "1060");

        let value = headers.get("x-ratelimit-reset");

        assert_eq!(value, Some("1060"));
    }

    #[test]
    fn get_returns_none_when_name_is_absent() {
        let headers = Headers::default();

        let value = headers.get("retry-after");

        assert_eq!(value, None);
    }

    #[test]
    fn insert_replaces_value_when_name_is_already_present() {
        let mut headers = Headers::default();
        headers.insert("Accept", "text/plain");

        headers.insert("accept", "application/json");

        assert_eq!(headers.get("accept"), Some("application/json"));
        assert_eq!(headers.iter().count(), 1);
    }

    #[test]
    fn insert_sensitive_marks_header_when_value_is_secret() {
        let mut headers = Headers::default();

        headers.insert_sensitive("APCA-API-SECRET-KEY", "test-secret");
        let header = headers.iter().next().unwrap();

        assert!(header.is_sensitive());
        assert_eq!(header.name(), "apca-api-secret-key");
        assert_eq!(header.value(), "test-secret");
    }

    #[test]
    fn fmt_redacts_value_when_header_is_sensitive() {
        let mut headers = Headers::default();
        headers.insert_sensitive("APCA-API-SECRET-KEY", "test-secret");

        let debug = format!("{headers:?}");

        assert!(debug.contains("apca-api-secret-key"));
        assert!(!debug.contains("test-secret"));
    }

    #[test]
    fn fmt_shows_value_when_header_is_not_sensitive() {
        let mut headers = Headers::default();
        headers.insert("content-type", "application/json");

        let debug = format!("{headers:?}");

        assert!(debug.contains("application/json"));
    }
}
