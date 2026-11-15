use super::test_server::{self, Recording, Reply};
use super::*;
use serde_json::{Value, json};

fn reply(status: &'static str, headers: &'static str) -> Reply {
    Reply::Response {
        status,
        headers,
        body: json!({}).to_string(),
    }
}

fn retrying(
    max_attempts: u32,
    rate_limit: Option<RateLimitHint>,
    script: Vec<Reply>,
) -> (RetryingTransport, Recording) {
    let (base_url, recording) = test_server::serve(script);
    let transport = Transport::new(base_url, HeaderMap::new()).unwrap();
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

fn one_minute(_: &HeaderMap, _: SystemTime) -> Option<Duration> {
    Some(Duration::from_secs(60))
}

#[test]
fn fetch_sends_get_with_query_when_transport_has_no_default_headers() {
    let (transport, server) = retrying(1, None, vec![reply("200 OK", "")]);
    let request = transport
        .get(&["series", "observations"])
        .unwrap()
        .query(&[("series_id", "GDP"), ("api_key", "k&1")]);

    let value: Value = transport.fetch(request).unwrap();
    let request = server.request();

    assert_eq!(value, json!({}));
    assert_eq!(request.method, "GET");
    assert_eq!(
        request.target,
        "/series/observations?series_id=GDP&api_key=k%261"
    );
    assert!(!request.headers.keys().any(|name| name.starts_with("apca")));
}

#[test]
fn fetch_backs_off_when_rate_limit_hint_is_absent() {
    let (transport, server) = retrying(
        2,
        None,
        vec![
            reply(
                "429 Too Many Requests",
                "X-RateLimit-Reset: 99999999999\r\n",
            ),
            reply("200 OK", ""),
        ],
    );

    let result = transport.fetch::<Value>(transport.get(&["series"]).unwrap());
    let requests = server.requests();

    assert!(result.is_ok());
    assert_eq!(requests.len(), 2);
}

#[test]
fn fetch_gives_up_when_rate_limit_hint_exceeds_max_delay() {
    let (transport, server) = retrying(
        3,
        Some(one_minute),
        vec![reply("429 Too Many Requests", "")],
    );

    let error = transport
        .fetch::<Value>(transport.get(&["series"]).unwrap())
        .unwrap_err();
    let requests = server.requests();

    assert_eq!(error, ClientError::HttpStatus { status: 429 });
    assert_eq!(requests.len(), 1);
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
