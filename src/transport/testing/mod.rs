pub(crate) mod scripted;
pub(crate) mod server;

use std::collections::HashMap;

use serde::Serialize;
use url::Url;

use super::{Headers, HttpMethod, Request};

pub(crate) fn query_pairs<Q: Serialize + ?Sized>(query: &Q) -> Vec<(String, String)> {
    let url = Url::parse("http://localhost/").unwrap();
    Request::new(HttpMethod::Get, url, Headers::default())
        .query(query)
        .unwrap()
        .into_http()
        .url
        .query_pairs()
        .into_owned()
        .collect()
}

pub(crate) struct RecordedRequest {
    pub(crate) method: String,
    pub(crate) target: String,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
}

impl RecordedRequest {
    pub(crate) fn path(&self) -> &str {
        self.target.split('?').next().unwrap()
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        Url::parse(&format!("http://localhost{}", self.target))
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect()
    }

    pub(crate) fn param(&self, name: &str) -> Option<String> {
        self.query()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }
}
