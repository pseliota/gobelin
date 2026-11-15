use serde_json::{Value, json};

use super::testing::scripted::{Recording, Reply, executor, interrupted, response, timed_out};
use super::*;

const BASE_URL: &str = "https://example.invalid/v2/";

fn ok() -> Reply {
    response(200, &[], "{}")
}

fn transport(script: Vec<Reply>) -> (Transport, Recording) {
    let (executor, recording) = executor(script);
    let mut headers = Headers::default();
    headers.insert_sensitive("APCA-API-KEY-ID", "test-key");
    let base_url = Url::parse(BASE_URL).unwrap();
    (Transport::new(base_url, headers, executor), recording)
}

fn retrying(
    max_attempts: u32,
    rate_limit: Option<RateLimitHint>,
    script: Vec<Reply>,
) -> (RetryingTransport, Recording) {
    let (transport, recording) = transport(script);
    let policy = RetryPolicy {
        max_attempts,
        base_delay: Duration::ZERO,
        max_delay: Duration::ZERO,
    };
    (
        RetryingTransport::new(transport, policy, rate_limit),
        recording,
    )
}

fn one_minute(_: &Headers, _: SystemTime) -> Option<Duration> {
    Some(Duration::from_secs(60))
}

fn retry_after(headers: &Headers, _: SystemTime) -> Option<Duration> {
    let seconds = headers.get("retry-after")?.parse().ok()?;
    Some(Duration::from_secs(seconds))
}

#[test]
fn request_appends_encoded_segments_when_base_url_has_path() {
    let (transport, _) = transport(vec![]);

    let request = transport.request(HttpMethod::Get, &["assets", "BTC/USD?x=1#f"]);

    assert_eq!(
        request.unwrap().into_http().url.as_str(),
        "https://example.invalid/v2/assets/BTC%2FUSD%3Fx=1%23f"
    );
}

#[test]
fn request_rejects_path_when_segment_is_empty_or_dotted() {
    let cases = ["", ".", ".."];
    let (transport, _) = transport(vec![]);

    for segment in cases {
        let error = transport
            .request(HttpMethod::Get, &["orders", segment])
            .err();

        assert_eq!(error, Some(ClientError::InvalidPath), "{segment:?}");
    }
}

#[test]
fn send_adds_default_headers_when_request_is_executed() {
    let (transport, recording) = transport(vec![ok()]);
    let request = transport.request(HttpMethod::Delete, &["orders"]).unwrap();

    transport.send(request, status::OK).unwrap();
    let request = recording.request();

    assert_eq!(request.method, "DELETE");
    assert_eq!(request.target, "/v2/orders");
    assert_eq!(request.headers["apca-api-key-id"], "test-key");
}

#[test]
fn send_returns_response_when_status_is_expected() {
    let (transport, _) = transport(vec![response(204, &[], "")]);
    let request = transport.request(HttpMethod::Delete, &["orders"]).unwrap();

    let response = transport.send(request, status::NO_CONTENT).unwrap();

    assert_eq!(response.status, 204);
    assert!(response.body.is_empty());
}

#[test]
fn send_reports_status_when_status_is_unexpected() {
    let (transport, _) = transport(vec![ok()]);
    let request = transport.request(HttpMethod::Delete, &["orders"]).unwrap();

    let error = transport.send(request, status::NO_CONTENT).unwrap_err();

    assert_eq!(error, ClientError::HttpStatus { status: 200 });
}

#[test]
fn send_maps_failure_to_client_error_when_executor_fails() {
    let cases = [
        (interrupted(), ClientError::Transport { timed_out: false }),
        (timed_out(), ClientError::Transport { timed_out: true }),
        (
            Err(TransportFailure::Unsendable),
            ClientError::InvalidRequest,
        ),
    ];

    for (reply, expected) in cases {
        let (transport, _) = transport(vec![reply]);
        let request = transport.request(HttpMethod::Post, &["orders"]).unwrap();

        let error = transport.send(request, status::OK).unwrap_err();

        assert_eq!(error, expected);
    }
}

#[test]
fn json_decodes_body_when_status_is_ok() {
    let (transport, _) = transport(vec![response(200, &[], r#"{"id":"order-1"}"#)]);
    let request = transport.request(HttpMethod::Get, &["orders"]).unwrap();

    let value: Value = transport.json(request).unwrap();

    assert_eq!(value, json!({"id": "order-1"}));
}

#[test]
fn json_reports_invalid_response_when_body_is_not_json() {
    let (transport, _) = transport(vec![response(200, &[], "not json")]);
    let request = transport.request(HttpMethod::Get, &["orders"]).unwrap();

    let error = transport.json::<Value>(request).unwrap_err();

    assert_eq!(error, ClientError::InvalidResponse);
}

#[test]
fn get_rejects_path_when_segment_is_dotted() {
    let (transport, _) = retrying(1, None, vec![]);

    let error = transport.get(&["series", ".."]).err();

    assert_eq!(error, Some(ClientError::InvalidPath));
}

#[test]
fn fetch_sends_get_with_query_and_default_headers_when_request_is_built() {
    let (transport, recording) = retrying(1, None, vec![ok()]);
    let request = transport
        .get(&["series", "observations"])
        .unwrap()
        .query(&[("series_id", "GDP"), ("api_key", "k&1")])
        .unwrap();

    let value: Value = transport.fetch(request).unwrap();
    let request = recording.request();

    assert_eq!(value, json!({}));
    assert_eq!(request.method, "GET");
    assert_eq!(
        request.target,
        "/v2/series/observations?series_id=GDP&api_key=k%261"
    );
    assert_eq!(request.headers["apca-api-key-id"], "test-key");
    assert!(request.body.is_empty());
}

#[test]
fn fetch_resends_identical_request_when_first_attempt_is_interrupted() {
    let (transport, recording) = retrying(2, None, vec![interrupted(), ok()]);
    let request = transport
        .get(&["series"])
        .unwrap()
        .query(&[("series_id", "GDP")])
        .unwrap();

    let result = transport.fetch::<Value>(request);
    let requests = recording.requests();

    assert!(result.is_ok());
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].target, requests[1].target);
    assert_eq!(requests[0].headers, requests[1].headers);
}

#[test]
fn fetch_does_not_retry_when_request_is_unsendable() {
    let (transport, recording) = retrying(3, None, vec![Err(TransportFailure::Unsendable)]);

    let error = transport
        .fetch::<Value>(transport.get(&["series"]).unwrap())
        .unwrap_err();
    let requests = recording.requests();

    assert_eq!(error, ClientError::InvalidRequest);
    assert_eq!(requests.len(), 1);
}

#[test]
fn fetch_does_not_retry_when_body_is_not_json() {
    let (transport, recording) = retrying(3, None, vec![response(200, &[], "not json")]);

    let error = transport
        .fetch::<Value>(transport.get(&["series"]).unwrap())
        .unwrap_err();
    let requests = recording.requests();

    assert_eq!(error, ClientError::InvalidResponse);
    assert_eq!(requests.len(), 1);
}

#[test]
fn fetch_backs_off_when_rate_limit_hint_is_absent() {
    let (transport, recording) = retrying(
        2,
        None,
        vec![
            response(429, &[("X-RateLimit-Reset", "99999999999")], "{}"),
            ok(),
        ],
    );

    let result = transport.fetch::<Value>(transport.get(&["series"]).unwrap());
    let requests = recording.requests();

    assert!(result.is_ok());
    assert_eq!(requests.len(), 2);
}

#[test]
fn fetch_gives_up_when_rate_limit_hint_exceeds_max_delay() {
    let (transport, recording) = retrying(3, Some(one_minute), vec![response(429, &[], "{}")]);

    let error = transport
        .fetch::<Value>(transport.get(&["series"]).unwrap())
        .unwrap_err();
    let requests = recording.requests();

    assert_eq!(error, ClientError::HttpStatus { status: 429 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn fetch_passes_response_headers_to_hint_when_rate_limited() {
    let (transport, recording) = retrying(
        3,
        Some(retry_after),
        vec![response(429, &[("Retry-After", "0")], "{}"), ok()],
    );

    let result = transport.fetch::<Value>(transport.get(&["series"]).unwrap());
    let requests = recording.requests();

    assert!(result.is_ok());
    assert_eq!(requests.len(), 2);
}

#[test]
fn fmt_names_no_provider_when_displaying_any_error() {
    let errors = [
        ClientError::InvalidCredentials,
        ClientError::Initialization,
        ClientError::InvalidPath,
        ClientError::InvalidRequest,
        ClientError::Transport { timed_out: true },
        ClientError::Transport { timed_out: false },
        ClientError::HttpStatus { status: 503 },
        ClientError::InvalidResponse,
    ];

    for error in errors {
        let message = error.to_string();

        assert!(!message.contains("Alpaca"), "{message}");
        assert!(!message.contains("header"), "{message}");
    }
}
