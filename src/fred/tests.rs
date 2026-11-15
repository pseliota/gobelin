use std::time::Duration;

use super::models::fixtures::{OBSERVATIONS, SEARCH, SERIES};
use super::models::{
    AggregationMethod, FilterVariable, Frequency, ObservationsRequest, SearchOrderBy,
    SearchRequest, SearchType, SortOrder, Units,
};
use super::*;
use crate::transport::testing::scripted::{
    Recording, Reply, executor, interrupted, response, timed_out,
};

const API_KEY: &str = "fred-secret-0123456789abcdef";

fn reply(status: u16, body: &str) -> Reply {
    response(status, &[], body)
}

fn retries(max_attempts: u32) -> FredConfig {
    FredConfig {
        retry: RetryPolicy {
            max_attempts,
            base_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
        },
        transport: TransportConfig::default(),
    }
}

fn server(config: FredConfig, script: Vec<Reply>) -> (FredClient, Recording) {
    let (executor, recording) = executor(script);
    let client = FredClient::with_executor(API_KEY, config, executor).unwrap();
    (client, recording)
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn observations_request(series_id: &str) -> ObservationsRequest {
    ObservationsRequest {
        series_id: series_id.into(),
        ..Default::default()
    }
}

fn search_request(search_text: &str) -> SearchRequest {
    SearchRequest {
        search_text: search_text.into(),
        ..Default::default()
    }
}

fn assert_key_absent(error: &ClientError) {
    let formatted = format!("{error} {error:?}");
    assert!(!formatted.contains(API_KEY), "{formatted}");
}

#[test]
fn new_uses_fred_endpoint_when_key_is_valid() {
    let client = FredClient::new(API_KEY, FredConfig::default());

    let client = client.unwrap();

    assert_eq!(
        client.transport.base_url().as_str(),
        "https://api.stlouisfed.org/"
    );
}

#[test]
fn new_rejects_credentials_when_key_is_blank() {
    let result = FredClient::new(" \t", FredConfig::default());

    assert!(matches!(result, Err(ClientError::InvalidCredentials)));
}

#[test]
fn get_series_sends_key_and_json_format_when_fetching_series() {
    let (client, server) = server(retries(1), vec![reply(200, SERIES)]);

    client.get_series("GNPCA").unwrap();
    let request = server.request();

    assert_eq!(request.method, "GET");
    assert_eq!(request.path(), "/fred/series");
    assert_eq!(
        request.query(),
        pairs(&[
            ("series_id", "GNPCA"),
            ("api_key", API_KEY),
            ("file_type", "json"),
        ])
    );
}

#[test]
fn get_series_sends_no_auth_headers_when_fetching_series() {
    let (client, server) = server(retries(1), vec![reply(200, SERIES)]);

    client.get_series("GNPCA").unwrap();
    let request = server.request();

    assert!(!request.headers.contains_key("authorization"));
    assert!(
        request
            .headers
            .values()
            .all(|value| !value.contains(API_KEY))
    );
}

#[test]
fn get_series_decodes_first_series_when_response_is_fixture() {
    let (client, server) = server(retries(1), vec![reply(200, SERIES)]);

    let series = client.get_series("GNPCA").unwrap();
    server.request();

    assert_eq!(series.id, "GNPCA");
    assert_eq!(series.title, "Real Gross National Product");
    assert_eq!(series.popularity, Some(39));
    assert_eq!(series.notes.as_deref(), Some("BEA Account Code: A001RX1"));
}

#[test]
fn get_series_returns_invalid_response_when_seriess_is_empty() {
    let body = r#"{"realtime_start":"2013-08-14","realtime_end":"2013-08-14","seriess":[]}"#;
    let (client, server) = server(retries(3), vec![reply(200, body)]);

    let error = client.get_series("GNPCA").unwrap_err();
    server.request();

    assert_eq!(error, ClientError::InvalidResponse);
}

#[test]
fn get_series_rejects_request_when_series_id_is_blank() {
    let (client, server) = server(retries(3), vec![]);

    let error = client.get_series(" ").unwrap_err();
    let requests = server.requests();

    assert_eq!(error, ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn get_series_retries_when_status_is_429() {
    let (client, server) = server(retries(3), vec![reply(429, "{}"), reply(200, SERIES)]);

    let series = client.get_series("GNPCA").unwrap();
    let requests = server.requests();

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].target, requests[1].target);
    assert_eq!(series.id, "GNPCA");
}

#[test]
fn get_series_retries_when_status_is_500() {
    let (client, server) = server(retries(3), vec![reply(500, "{}"), reply(200, SERIES)]);

    let series = client.get_series("GNPCA").unwrap();
    let requests = server.requests();

    assert_eq!(requests.len(), 2);
    assert_eq!(series.id, "GNPCA");
}

#[test]
fn get_series_does_not_retry_when_status_is_400() {
    let (client, server) = server(retries(3), vec![reply(400, "{}")]);

    let error = client.get_series("GNPCA").unwrap_err();
    let requests = server.requests();

    assert_eq!(error, ClientError::HttpStatus { status: 400 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_series_sends_one_request_when_policy_is_none() {
    let config = FredConfig {
        retry: RetryPolicy::none(),
        ..Default::default()
    };
    let (client, server) = server(config, vec![reply(500, "{}")]);

    let error = client.get_series("GNPCA").unwrap_err();
    let requests = server.requests();

    assert_eq!(error, ClientError::HttpStatus { status: 500 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_series_omits_key_from_error_when_status_is_400() {
    let (client, server) = server(retries(1), vec![reply(400, "{}")]);

    let error = client.get_series("GNPCA").unwrap_err();
    server.request();

    assert_key_absent(&error);
}

#[test]
fn get_series_omits_key_from_error_when_request_is_interrupted() {
    let (client, _server) = server(retries(1), vec![interrupted()]);

    let error = client.get_series("GNPCA").unwrap_err();

    assert!(matches!(error, ClientError::Transport { .. }));
    assert_key_absent(&error);
}

#[test]
fn get_series_omits_key_from_error_when_request_times_out() {
    let (client, _server) = server(retries(1), vec![timed_out()]);

    let error = client.get_series("GNPCA").unwrap_err();

    assert_eq!(error, ClientError::Transport { timed_out: true });
    assert_key_absent(&error);
}

#[test]
fn get_observations_sends_key_and_json_format_when_fetching_observations() {
    let (client, server) = server(retries(1), vec![reply(200, OBSERVATIONS)]);

    client
        .get_observations(&observations_request("GNPCA"))
        .unwrap();
    let request = server.request();

    assert_eq!(request.method, "GET");
    assert_eq!(request.path(), "/fred/series/observations");
    assert_eq!(
        request.query(),
        pairs(&[
            ("series_id", "GNPCA"),
            ("api_key", API_KEY),
            ("file_type", "json"),
        ])
    );
}

#[test]
fn get_observations_encodes_request_fields_when_request_is_populated() {
    let (client, server) = server(retries(1), vec![reply(200, OBSERVATIONS)]);
    let request = ObservationsRequest {
        series_id: "GNPCA".into(),
        observation_start: Some("1990-01-01".into()),
        observation_end: Some("2000-01-01".into()),
        realtime_start: Some("2005-01-01".into()),
        realtime_end: Some("2005-12-31".into()),
        vintage_dates: vec!["2005-01-01".into(), "2005-06-01".into()],
        limit: Some(10),
        offset: Some(20),
        sort_order: Some(SortOrder::Desc),
        units: Some(Units::Change),
        frequency: Some(Frequency::Annual),
        aggregation_method: Some(AggregationMethod::Sum),
    };

    client.get_observations(&request).unwrap();
    let recorded = server.request();

    assert_eq!(
        recorded.query(),
        pairs(&[
            ("series_id", "GNPCA"),
            ("observation_start", "1990-01-01"),
            ("observation_end", "2000-01-01"),
            ("realtime_start", "2005-01-01"),
            ("realtime_end", "2005-12-31"),
            ("vintage_dates", "2005-01-01,2005-06-01"),
            ("limit", "10"),
            ("offset", "20"),
            ("sort_order", "desc"),
            ("units", "chg"),
            ("frequency", "a"),
            ("aggregation_method", "sum"),
            ("api_key", API_KEY),
            ("file_type", "json"),
        ])
    );
}

#[test]
fn get_observations_decodes_page_when_response_is_fixture() {
    let (client, server) = server(retries(1), vec![reply(200, OBSERVATIONS)]);

    let page = client
        .get_observations(&observations_request("GNPCA"))
        .unwrap();
    server.request();

    assert_eq!(page.count, 84);
    assert_eq!(page.observations.len(), 2);
    assert_eq!(page.observations[0].numeric_value(), Some(1065.9));
    assert_eq!(page.next_offset(), Some(2));
}

#[test]
fn get_observations_rejects_request_when_series_id_is_blank() {
    let (client, server) = server(retries(3), vec![]);

    let error = client
        .get_observations(&observations_request(""))
        .unwrap_err();
    let requests = server.requests();

    assert_eq!(error, ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn search_series_sends_key_and_json_format_when_searching() {
    let (client, server) = server(retries(1), vec![reply(200, SEARCH)]);

    client
        .search_series(&search_request("money stock"))
        .unwrap();
    let request = server.request();

    assert_eq!(request.method, "GET");
    assert_eq!(request.path(), "/fred/series/search");
    assert_eq!(
        request.query(),
        pairs(&[
            ("search_text", "money stock"),
            ("api_key", API_KEY),
            ("file_type", "json"),
        ])
    );
}

#[test]
fn search_series_encodes_request_fields_when_request_is_populated() {
    let (client, server) = server(retries(1), vec![reply(200, SEARCH)]);
    let request = SearchRequest {
        search_text: "money & stock".into(),
        search_type: Some(SearchType::SeriesId),
        limit: Some(5),
        offset: Some(10),
        order_by: Some(SearchOrderBy::LastUpdated),
        sort_order: Some(SortOrder::Asc),
        filter_variable: Some(FilterVariable::SeasonalAdjustment),
        filter_value: Some("Seasonally Adjusted".into()),
        tag_names: vec!["usa".into(), "m2".into()],
        exclude_tag_names: vec!["discontinued".into()],
        realtime_start: Some("2017-08-01".into()),
        realtime_end: Some("2017-08-02".into()),
    };

    client.search_series(&request).unwrap();
    let recorded = server.request();

    assert_eq!(
        recorded.query(),
        pairs(&[
            ("search_text", "money & stock"),
            ("search_type", "series_id"),
            ("limit", "5"),
            ("offset", "10"),
            ("order_by", "last_updated"),
            ("sort_order", "asc"),
            ("filter_variable", "seasonal_adjustment"),
            ("filter_value", "Seasonally Adjusted"),
            ("tag_names", "usa;m2"),
            ("exclude_tag_names", "discontinued"),
            ("realtime_start", "2017-08-01"),
            ("realtime_end", "2017-08-02"),
            ("api_key", API_KEY),
            ("file_type", "json"),
        ])
    );
}

#[test]
fn search_series_decodes_page_when_response_is_fixture() {
    let (client, server) = server(retries(1), vec![reply(200, SEARCH)]);

    let page = client
        .search_series(&search_request("money stock"))
        .unwrap();
    server.request();

    assert_eq!(page.count, 32);
    assert_eq!(page.series.len(), 1);
    assert_eq!(page.series[0].id, "MSIM2");
    assert_eq!(page.series[0].group_popularity, Some(33));
    assert_eq!(page.next_offset(), Some(1));
}

#[test]
fn search_series_rejects_request_when_search_text_is_blank() {
    let (client, server) = server(retries(3), vec![]);

    let error = client.search_series(&search_request("\n")).unwrap_err();
    let requests = server.requests();

    assert_eq!(error, ClientError::InvalidRequest);
    assert!(requests.is_empty());
}
