use std::time::Duration;

use url::Url;

use super::*;
use crate::transport::testing::server::{self, Reply};

fn executor() -> ReqwestExecutor {
    ReqwestExecutor::without_proxy(&TransportConfig::default()).unwrap()
}

fn request(method: HttpMethod, url: Url) -> HttpRequest {
    HttpRequest {
        method,
        url,
        headers: Headers::default(),
        body: None,
    }
}

fn reply(status: &'static str, headers: &'static str, body: &str) -> Reply {
    Reply::Response {
        status,
        headers,
        body: body.to_owned(),
    }
}

#[test]
fn execute_sends_method_target_and_headers_when_request_is_get() {
    let (base_url, server) = server::serve(vec![reply("200 OK", "", "{}")]);
    let mut request = request(
        HttpMethod::Get,
        base_url.join("v2/account?nested=true").unwrap(),
    );
    request
        .headers
        .insert_sensitive("APCA-API-KEY-ID", "test-key");

    executor().execute(request).unwrap();
    let recorded = server.request();

    assert_eq!(recorded.method, "GET");
    assert_eq!(recorded.target, "/v2/account?nested=true");
    assert_eq!(recorded.headers["apca-api-key-id"], "test-key");
    assert!(recorded.body.is_empty());
}

#[test]
fn execute_sends_body_when_request_has_body() {
    let cases = [
        (HttpMethod::Post, "POST"),
        (HttpMethod::Patch, "PATCH"),
        (HttpMethod::Delete, "DELETE"),
    ];

    for (method, expected) in cases {
        let (base_url, server) = server::serve(vec![reply("200 OK", "", "{}")]);
        let mut request = request(method, base_url.join("v2/orders").unwrap());
        request.headers.insert("content-type", "application/json");
        request.body = Some(br#"{"qty":"2"}"#.to_vec());

        executor().execute(request).unwrap();
        let recorded = server.request();

        assert_eq!(recorded.method, expected);
        assert_eq!(recorded.headers["content-type"], "application/json");
        assert_eq!(recorded.body, br#"{"qty":"2"}"#);
    }
}

#[test]
fn execute_sends_user_agent_when_config_names_one() {
    let (base_url, server) = server::serve(vec![reply("200 OK", "", "{}")]);
    let config = TransportConfig {
        user_agent: "gobelin-test/1.0".into(),
        ..Default::default()
    };
    let executor = ReqwestExecutor::without_proxy(&config).unwrap();

    executor
        .execute(request(HttpMethod::Get, base_url))
        .unwrap();
    let recorded = server.request();

    assert_eq!(recorded.headers["user-agent"], "gobelin-test/1.0");
}

#[test]
fn execute_returns_status_headers_and_body_when_server_responds() {
    let (base_url, server) = server::serve(vec![reply(
        "429 Too Many Requests",
        "X-RateLimit-Reset: 1060\r\n",
        r#"{"message":"slow down"}"#,
    )]);

    let response = executor()
        .execute(request(HttpMethod::Get, base_url))
        .unwrap();
    server.request();

    assert_eq!(response.status, 429);
    assert_eq!(response.headers.get("x-ratelimit-reset"), Some("1060"));
    assert_eq!(response.body, br#"{"message":"slow down"}"#);
}

#[test]
fn execute_returns_redirect_unfollowed_when_server_returns_location() {
    let (base_url, server) = server::serve(vec![reply(
        "302 Found",
        "Location: https://example.invalid/\r\n",
        "",
    )]);

    let response = executor()
        .execute(request(HttpMethod::Get, base_url))
        .unwrap();
    let requests = server.requests();

    assert_eq!(response.status, 302);
    assert_eq!(
        response.headers.get("location"),
        Some("https://example.invalid/")
    );
    assert_eq!(requests.len(), 1);
}

#[test]
fn execute_reports_interruption_when_server_closes_without_response() {
    let (base_url, server) = server::serve(vec![Reply::Close]);

    let result = executor().execute(request(HttpMethod::Get, base_url));
    let requests = server.requests();

    assert_eq!(
        result,
        Err(TransportFailure::Interrupted { timed_out: false })
    );
    assert_eq!(requests.len(), 1);
}

#[test]
fn execute_reports_timeout_when_server_stalls() {
    let (base_url, server) = server::serve(vec![Reply::Stall]);
    let config = TransportConfig {
        timeout: Duration::from_millis(200),
        ..Default::default()
    };
    let executor = ReqwestExecutor::without_proxy(&config).unwrap();

    let result = executor.execute(request(HttpMethod::Get, base_url));
    drop(executor);
    server.request();

    assert_eq!(
        result,
        Err(TransportFailure::Interrupted { timed_out: true })
    );
}

#[test]
fn execute_reports_unsendable_when_header_value_has_newline() {
    let url = Url::parse("http://127.0.0.1:9/").unwrap();
    let mut request = request(HttpMethod::Get, url);
    request.headers.insert("x-test", "line\nbreak");

    let result = executor().execute(request);

    assert_eq!(result, Err(TransportFailure::Unsendable));
}

#[test]
fn execute_reports_unsendable_when_header_name_has_space() {
    let url = Url::parse("http://127.0.0.1:9/").unwrap();
    let mut request = request(HttpMethod::Get, url);
    request.headers.insert("x test", "value");

    let result = executor().execute(request);

    assert_eq!(result, Err(TransportFailure::Unsendable));
}

#[test]
fn new_reports_initialization_when_user_agent_has_newline() {
    let config = TransportConfig {
        user_agent: "gobelin\nagent".into(),
        ..Default::default()
    };

    let error = ReqwestExecutor::new(&config).err();

    assert_eq!(error, Some(ClientError::Initialization));
}
