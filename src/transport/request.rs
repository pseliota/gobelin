use serde::Serialize;
use url::Url;

use super::{
    ClientError,
    http::{Headers, HttpMethod, HttpRequest},
};

pub(crate) struct Request(HttpRequest);

impl Request {
    pub(super) fn new(method: HttpMethod, url: Url, headers: Headers) -> Self {
        Self(HttpRequest {
            method,
            url,
            headers,
            body: None,
        })
    }

    pub(crate) fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Result<Self, ClientError> {
        {
            let mut pairs = self.0.url.query_pairs_mut();
            query
                .serialize(serde_urlencoded::Serializer::new(&mut pairs))
                .map_err(|_| ClientError::InvalidRequest)?;
        }
        if self.0.url.query() == Some("") {
            self.0.url.set_query(None);
        }
        Ok(self)
    }

    pub(crate) fn json<B: Serialize + ?Sized>(mut self, body: &B) -> Result<Self, ClientError> {
        let body = serde_json::to_vec(body).map_err(|_| ClientError::InvalidRequest)?;
        self.0.headers.insert("content-type", "application/json");
        self.0.body = Some(body);
        Ok(self)
    }

    pub(super) fn into_http(self) -> HttpRequest {
        self.0
    }

    pub(super) fn to_http(&self) -> HttpRequest {
        self.0.clone()
    }
}

pub(crate) struct GetRequest(Request);

impl GetRequest {
    pub(super) fn new(url: Url, headers: Headers) -> Self {
        Self(Request::new(HttpMethod::Get, url, headers))
    }

    pub(crate) fn query<Q: Serialize + ?Sized>(self, query: &Q) -> Result<Self, ClientError> {
        self.0.query(query).map(Self)
    }

    pub(super) fn to_http(&self) -> HttpRequest {
        self.0.to_http()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::Serializer;

    use super::*;

    struct Unserializable;

    impl Serialize for Unserializable {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("unserializable"))
        }
    }

    fn request(method: HttpMethod, url: &str) -> Request {
        Request::new(method, Url::parse(url).unwrap(), Headers::default())
    }

    #[test]
    fn query_appends_encoded_pairs_when_url_has_no_query() {
        let request = request(HttpMethod::Get, "https://example.invalid/series");

        let request = request.query(&[("series_id", "GDP"), ("api_key", "k&1")]);

        assert_eq!(
            request.unwrap().into_http().url.as_str(),
            "https://example.invalid/series?series_id=GDP&api_key=k%261"
        );
    }

    #[test]
    fn query_keeps_existing_pairs_when_called_twice() {
        let request = request(HttpMethod::Get, "https://example.invalid/bars")
            .query(&[("feed", "iex")])
            .unwrap();

        let request = request.query(&[("symbols", "AAPL,MSFT")]);

        assert_eq!(
            request.unwrap().into_http().url.query(),
            Some("feed=iex&symbols=AAPL%2CMSFT")
        );
    }

    #[test]
    fn query_leaves_url_bare_when_query_is_empty() {
        let request = request(HttpMethod::Get, "https://example.invalid/account");

        let request = request.query(&BTreeMap::<String, String>::new());

        assert_eq!(
            request.unwrap().into_http().url.as_str(),
            "https://example.invalid/account"
        );
    }

    #[test]
    fn query_reports_invalid_request_when_query_is_not_a_pair_list() {
        let request = request(HttpMethod::Get, "https://example.invalid/account");

        let error = request.query("not-pairs").err();

        assert_eq!(error, Some(ClientError::InvalidRequest));
    }

    #[test]
    fn json_sets_body_and_content_type_when_body_serializes() {
        let request = request(HttpMethod::Post, "https://example.invalid/orders");

        let request = request.json(&[("symbol", "AAPL")]).unwrap().into_http();

        assert_eq!(
            request.body.as_deref(),
            Some(&br#"[["symbol","AAPL"]]"#[..])
        );
        assert_eq!(
            request.headers.get("content-type"),
            Some("application/json")
        );
    }

    #[test]
    fn json_reports_invalid_request_when_body_does_not_serialize() {
        let request = request(HttpMethod::Post, "https://example.invalid/orders");

        let error = request.json(&Unserializable).err();

        assert_eq!(error, Some(ClientError::InvalidRequest));
    }

    #[test]
    fn new_builds_get_without_body_when_get_request_is_created() {
        let url = Url::parse("https://example.invalid/series").unwrap();

        let request = GetRequest::new(url, Headers::default()).to_http();

        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.body, None);
    }
}
