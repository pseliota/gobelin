use super::models::{Adjustment, BarsRequest};
use super::*;
use crate::alpaca::models::SortDirection;
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread::JoinHandle,
    time::{Duration, Instant},
};

struct RecordedRequest {
    method: String,
    target: String,
    headers: HashMap<String, String>,
}

impl RecordedRequest {
    fn path(&self) -> &str {
        self.target.split('?').next().unwrap()
    }

    fn query(&self) -> Vec<(String, String)> {
        Url::parse(&format!("http://localhost{}", self.target))
            .unwrap()
            .query_pairs()
            .into_owned()
            .collect()
    }

    fn param(&self, name: &str) -> Option<String> {
        self.query()
            .into_iter()
            .find_map(|(key, value)| (key == name).then_some(value))
    }
}

/// A scripted answer to one request.
enum Reply {
    Response {
        status: &'static str,
        body: String,
        headers: &'static str,
    },
    /// Closes the connection without answering.
    Close,
}

fn reply(status: &'static str, body: Value) -> Reply {
    reply_with_headers(status, body, "")
}

fn reply_with_headers(status: &'static str, body: Value, headers: &'static str) -> Reply {
    Reply::Response {
        status,
        body: body.to_string(),
        headers,
    }
}

fn retries(max_attempts: u32) -> DataClientConfig {
    DataClientConfig {
        feed: None,
        retry: RetryPolicy {
            max_attempts,
            base_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
        },
    }
}

fn feed(feed: Feed) -> DataClientConfig {
    DataClientConfig {
        feed: Some(feed),
        ..retries(1)
    }
}

// Bounded loopback exchanges, one connection per scripted reply;
// no credentials, proxies, or external services.
fn server(
    config: DataClientConfig,
    script: Vec<Reply>,
) -> (AlpacaDataClient, JoinHandle<Vec<RecordedRequest>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        script
            .into_iter()
            .map(|reply| exchange(&listener, reply))
            .collect()
    });
    let base_url = Url::parse(&format!("http://{address}/")).unwrap();
    let client = AlpacaDataClient::build("test-key", "test-secret", base_url, config).unwrap();
    (client, handle)
}

fn exchange(listener: &TcpListener, reply: Reply) -> RecordedRequest {
    let mut stream = accept(listener);
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut buffer = [0; 4096];
        let count = stream.read(&mut buffer).unwrap();
        assert!(count > 0, "incomplete headers");
        bytes.extend_from_slice(&buffer[..count]);
        if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            break end + 4;
        }
        assert!(bytes.len() < 65536, "oversized headers");
    };
    let header_text = String::from_utf8(bytes[..header_end].to_vec()).unwrap();
    let mut lines = header_text.split("\r\n");
    let mut request_line = lines.next().unwrap().split_whitespace();
    let method = request_line.next().unwrap().to_owned();
    let target = request_line.next().unwrap().to_owned();
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    if let Reply::Response {
        status,
        body,
        headers,
    } = reply
    {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    }
    RecordedRequest {
        method,
        target,
        headers,
    }
}

fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(5);
    let stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < deadline, "client never connected");
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => panic!("accept failed: {error}"),
        }
    };
    stream.set_nonblocking(false).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    stream
}

fn bars_request(symbols: &[&str]) -> BarsRequest {
    BarsRequest {
        symbols: symbols.iter().map(|s| (*s).to_owned()).collect(),
        timeframe: "1Day".into(),
        ..Default::default()
    }
}

fn bar(close: f64) -> Value {
    json!({
        "t": "2022-08-17T08:58:00Z", "o": 172.81, "h": 172.81, "l": 172.78,
        "c": close, "v": 1002, "n": 20, "vw": 172.791417
    })
}

fn quote() -> Value {
    json!({
        "ap": 172.7, "as": 5, "ax": "Q", "bp": 172.6, "bs": 2, "bx": "Q",
        "c": ["R"], "t": "2022-08-17T10:18:27.052763263Z", "z": "C"
    })
}

fn trade() -> Value {
    json!({
        "c": ["@", "T"], "i": 1011, "p": 172.61, "s": 160,
        "t": "2022-08-17T10:18:24.114694956Z", "x": "Q", "z": "C"
    })
}

fn bars_page(close: f64, next_page_token: Option<&str>) -> Value {
    json!({"bars": {"AAPL": [bar(close)]}, "next_page_token": next_page_token})
}

fn latest_bars() -> Value {
    json!({"bars": {"AAPL": bar(172.79)}})
}

#[test]
fn new_uses_unversioned_data_endpoint_when_credentials_are_valid() {
    let client = AlpacaDataClient::new("test-key", "test-secret", DataClientConfig::default());

    let client = client.unwrap();

    assert_eq!(
        client.transport.base_url().as_str(),
        "https://data.alpaca.markets/"
    );
}

#[test]
fn new_rejects_credentials_when_key_is_blank() {
    let result = AlpacaDataClient::new(" ", "test-secret", DataClientConfig::default());

    assert!(matches!(result, Err(ClientError::InvalidCredentials)));
}

#[test]
fn get_stock_bars_authenticates_request_when_fetching_page() {
    let (client, server) = server(retries(1), vec![reply("200 OK", bars_page(172.79, None))]);

    let page = client
        .get_stock_bars(&bars_request(&["AAPL"]), None)
        .unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path(), "/v2/stocks/bars");
    assert_eq!(requests[0].headers["apca-api-key-id"], "test-key");
    assert_eq!(requests[0].headers["apca-api-secret-key"], "test-secret");
    assert_eq!(page.bars["AAPL"][0].close, 172.79);
    assert_eq!(page.next_page_token, None);
}

#[test]
fn get_stock_bars_encodes_request_fields_when_request_is_populated() {
    let (client, server) = server(retries(1), vec![reply("200 OK", bars_page(172.79, None))]);
    let request = BarsRequest {
        symbols: vec!["AAPL".into(), "BRK.B".into()],
        timeframe: "5Min".into(),
        start: Some("2024-01-02T09:30:00-05:00".into()),
        end: Some("2024-01-03".into()),
        limit: Some(1000),
        adjustment: vec![Adjustment::Split, Adjustment::SpinOff],
        asof: Some("2024-01-04".into()),
        currency: Some("EUR".into()),
        sort: Some(SortDirection::Desc),
    };

    client.get_stock_bars(&request, None).unwrap();
    let requests = server.join().unwrap();

    let expected = [
        ("symbols", "AAPL,BRK.B"),
        ("timeframe", "5Min"),
        ("start", "2024-01-02T09:30:00-05:00"),
        ("end", "2024-01-03"),
        ("limit", "1000"),
        ("adjustment", "split,spin-off"),
        ("asof", "2024-01-04"),
        ("currency", "EUR"),
        ("sort", "desc"),
    ]
    .map(|(key, value)| (key.to_owned(), value.to_owned()));
    assert_eq!(requests[0].query(), expected);
    assert!(!requests[0].target.contains("09:30"));
}

#[test]
fn get_stock_bars_sends_page_token_when_token_is_supplied() {
    let (client, server) = server(retries(1), vec![reply("200 OK", bars_page(172.79, None))]);

    client
        .get_stock_bars(&bars_request(&["AAPL"]), Some("token&1"))
        .unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].param("page_token").as_deref(), Some("token&1"));
    assert!(requests[0].target.contains("page_token=token%261"));
}

#[test]
fn get_stock_bars_omits_page_token_when_token_is_not_supplied() {
    let (client, server) = server(retries(1), vec![reply("200 OK", bars_page(172.79, None))]);

    client
        .get_stock_bars(&bars_request(&["AAPL"]), None)
        .unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].param("page_token"), None);
}

#[test]
fn get_stock_bars_sends_feed_when_feed_is_configured() {
    let (client, server) = server(
        feed(Feed::DelayedSip),
        vec![reply("200 OK", bars_page(172.79, None))],
    );

    client
        .get_stock_bars(&bars_request(&["AAPL"]), None)
        .unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].param("feed").as_deref(), Some("delayed_sip"));
}

#[test]
fn get_stock_bars_omits_feed_when_feed_is_not_configured() {
    let (client, server) = server(retries(1), vec![reply("200 OK", bars_page(172.79, None))]);

    client
        .get_stock_bars(&bars_request(&["AAPL"]), None)
        .unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].param("feed"), None);
}

#[test]
fn get_stock_bars_returns_empty_page_when_bars_are_null() {
    let (client, server) = server(
        retries(1),
        vec![reply(
            "200 OK",
            json!({"bars": null, "next_page_token": null}),
        )],
    );

    let page = client
        .get_stock_bars(&bars_request(&["AAPL"]), None)
        .unwrap();
    server.join().unwrap();

    assert!(page.bars.is_empty());
}

#[test]
fn get_stock_bars_rejects_request_when_symbols_are_empty() {
    let (client, server) = server(retries(3), vec![]);

    let result = client.get_stock_bars(&bars_request(&[]), None);
    let requests = server.join().unwrap();

    assert_eq!(result.unwrap_err(), ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn get_stock_bars_rejects_request_when_symbol_is_blank() {
    let (client, server) = server(retries(3), vec![]);

    let result = client.get_stock_bars(&bars_request(&["AAPL", " "]), None);
    let requests = server.join().unwrap();

    assert_eq!(result.unwrap_err(), ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn stock_bars_pages_follows_tokens_when_more_pages_exist() {
    let (client, server) = server(
        feed(Feed::Iex),
        vec![
            reply("200 OK", bars_page(1.0, Some("second"))),
            reply("200 OK", bars_page(2.0, Some("third"))),
            reply("200 OK", bars_page(3.0, None)),
        ],
    );
    let request = bars_request(&["AAPL"]);

    let pages: Vec<_> = client.stock_bars_pages(&request).collect();
    let requests = server.join().unwrap();

    let closes: Vec<f64> = pages
        .iter()
        .map(|page| page.as_ref().unwrap().bars["AAPL"][0].close)
        .collect();
    let tokens: Vec<_> = requests.iter().map(|r| r.param("page_token")).collect();
    assert_eq!(closes, [1.0, 2.0, 3.0]);
    assert_eq!(tokens, [None, Some("second".into()), Some("third".into())]);
    assert!(requests.iter().all(|r| r.path() == "/v2/stocks/bars"));
    assert!(
        requests
            .iter()
            .all(|r| r.param("symbols").as_deref() == Some("AAPL")
                && r.param("timeframe").as_deref() == Some("1Day")
                && r.param("feed").as_deref() == Some("iex"))
    );
}

#[test]
fn stock_bars_pages_stops_when_page_fails() {
    let (client, server) = server(
        retries(1),
        vec![
            reply("200 OK", bars_page(1.0, Some("second"))),
            reply("403 Forbidden", json!({})),
        ],
    );
    let request = bars_request(&["AAPL"]);

    let pages: Vec<_> = client.stock_bars_pages(&request).collect();
    let requests = server.join().unwrap();

    assert_eq!(pages.len(), 2);
    assert!(pages[0].is_ok());
    assert_eq!(pages[1], Err(ClientError::HttpStatus { status: 403 }));
    assert_eq!(requests.len(), 2);
}

#[test]
fn stock_bars_pages_sends_nothing_when_not_advanced() {
    let (client, server) = server(retries(1), vec![]);
    let request = bars_request(&["AAPL"]);

    let pages = client.stock_bars_pages(&request);
    drop(pages);
    let requests = server.join().unwrap();

    assert!(requests.is_empty());
}

#[test]
fn stock_bars_pages_yields_error_when_symbols_are_empty() {
    let (client, server) = server(retries(1), vec![]);
    let request = bars_request(&[]);

    let pages: Vec<_> = client.stock_bars_pages(&request).collect();
    let requests = server.join().unwrap();

    assert_eq!(pages, [Err(ClientError::InvalidRequest)]);
    assert!(requests.is_empty());
}

#[test]
fn get_latest_stock_bars_joins_symbols_when_several_are_requested() {
    let (client, server) = server(
        feed(Feed::Sip),
        vec![reply(
            "200 OK",
            json!({"bars": {"AAPL": bar(172.79), "TSLA": bar(914.3)}}),
        )],
    );

    let bars = client.get_latest_stock_bars(&["AAPL", "TSLA"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path(), "/v2/stocks/bars/latest");
    assert_eq!(requests[0].headers["apca-api-key-id"], "test-key");
    assert_eq!(requests[0].headers["apca-api-secret-key"], "test-secret");
    assert_eq!(
        requests[0].query(),
        [
            ("feed".to_owned(), "sip".to_owned()),
            ("symbols".to_owned(), "AAPL,TSLA".to_owned())
        ]
    );
    assert_eq!(bars.len(), 2);
    assert_eq!(bars["TSLA"].close, 914.3);
}

#[test]
fn get_latest_stock_bars_omits_feed_when_feed_is_not_configured() {
    let (client, server) = server(retries(1), vec![reply("200 OK", latest_bars())]);

    client.get_latest_stock_bars(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].target, "/v2/stocks/bars/latest?symbols=AAPL");
}

#[test]
fn get_latest_stock_bars_rejects_request_when_symbols_are_empty() {
    let (client, server) = server(retries(3), vec![]);

    let result = client.get_latest_stock_bars(&[]);
    let requests = server.join().unwrap();

    assert_eq!(result.unwrap_err(), ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn get_latest_stock_quotes_decodes_quotes_when_feed_is_configured() {
    let (client, server) = server(
        feed(Feed::Iex),
        vec![reply("200 OK", json!({"quotes": {"AAPL": quote()}}))],
    );

    let quotes = client.get_latest_stock_quotes(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path(), "/v2/stocks/quotes/latest");
    assert_eq!(requests[0].headers["apca-api-key-id"], "test-key");
    assert_eq!(requests[0].headers["apca-api-secret-key"], "test-secret");
    assert_eq!(requests[0].param("symbols").as_deref(), Some("AAPL"));
    assert_eq!(requests[0].param("feed").as_deref(), Some("iex"));
    assert_eq!(quotes["AAPL"].ask_price, 172.7);
}

#[test]
fn get_latest_stock_quotes_rejects_request_when_symbol_is_empty() {
    let (client, server) = server(retries(3), vec![]);

    let result = client.get_latest_stock_quotes(&["AAPL", ""]);
    let requests = server.join().unwrap();

    assert_eq!(result.unwrap_err(), ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn get_latest_stock_trades_decodes_trades_when_feed_is_configured() {
    let (client, server) = server(
        feed(Feed::Boats),
        vec![reply("200 OK", json!({"trades": {"AAPL": trade()}}))],
    );

    let trades = client.get_latest_stock_trades(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path(), "/v2/stocks/trades/latest");
    assert_eq!(requests[0].headers["apca-api-key-id"], "test-key");
    assert_eq!(requests[0].headers["apca-api-secret-key"], "test-secret");
    assert_eq!(requests[0].param("symbols").as_deref(), Some("AAPL"));
    assert_eq!(requests[0].param("feed").as_deref(), Some("boats"));
    assert_eq!(trades["AAPL"].id, 1011);
}

#[test]
fn get_latest_stock_trades_rejects_request_when_symbols_are_empty() {
    let (client, server) = server(retries(3), vec![]);

    let result = client.get_latest_stock_trades(&[]);
    let requests = server.join().unwrap();

    assert_eq!(result.unwrap_err(), ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn get_stock_snapshots_decodes_bare_object_when_keyed_by_symbol() {
    let (client, server) = server(
        feed(Feed::Overnight),
        vec![reply(
            "200 OK",
            json!({
                "AAPL": {
                    "latestTrade": trade(), "latestQuote": quote(), "minuteBar": bar(172.69),
                    "dailyBar": bar(173.03), "prevDailyBar": bar(173.19)
                },
                "TSLA": {"latestTrade": null}
            }),
        )],
    );

    let snapshots = client.get_stock_snapshots(&["AAPL", "TSLA"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests[0].method, "GET");
    assert_eq!(requests[0].path(), "/v2/stocks/snapshots");
    assert_eq!(requests[0].headers["apca-api-key-id"], "test-key");
    assert_eq!(requests[0].headers["apca-api-secret-key"], "test-secret");
    assert_eq!(requests[0].param("symbols").as_deref(), Some("AAPL,TSLA"));
    assert_eq!(requests[0].param("feed").as_deref(), Some("overnight"));
    let aapl = &snapshots["AAPL"];
    assert_eq!(aapl.latest_trade.as_ref().unwrap().price, 172.61);
    assert_eq!(aapl.latest_quote.as_ref().unwrap().bid_price, 172.6);
    assert_eq!(aapl.minute_bar.as_ref().unwrap().close, 172.69);
    assert_eq!(aapl.daily_bar.as_ref().unwrap().close, 173.03);
    assert_eq!(aapl.prev_daily_bar.as_ref().unwrap().close, 173.19);
    assert_eq!(snapshots["TSLA"].latest_trade, None);
    assert_eq!(snapshots["TSLA"].daily_bar, None);
}

#[test]
fn get_stock_snapshots_rejects_request_when_symbol_is_blank() {
    let (client, server) = server(retries(3), vec![]);

    let result = client.get_stock_snapshots(&["\t"]);
    let requests = server.join().unwrap();

    assert_eq!(result.unwrap_err(), ClientError::InvalidRequest);
    assert!(requests.is_empty());
}

#[test]
fn get_latest_stock_bars_retries_when_status_is_429() {
    let (client, server) = server(
        retries(3),
        vec![
            reply("429 Too Many Requests", json!({})),
            reply("200 OK", latest_bars()),
        ],
    );

    let bars = client.get_latest_stock_bars(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].target, requests[1].target);
    assert_eq!(requests[1].headers["apca-api-key-id"], "test-key");
    assert_eq!(bars["AAPL"].close, 172.79);
}

#[test]
fn get_latest_stock_bars_retries_when_status_is_503() {
    let (client, server) = server(
        retries(3),
        vec![
            reply("503 Service Unavailable", json!({})),
            reply("503 Service Unavailable", json!({})),
            reply("200 OK", latest_bars()),
        ],
    );

    let bars = client.get_latest_stock_bars(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests.len(), 3);
    assert_eq!(bars["AAPL"].close, 172.79);
}

#[test]
fn get_latest_stock_bars_retries_when_connection_closes_without_response() {
    let (client, server) = server(
        retries(2),
        vec![Reply::Close, reply("200 OK", latest_bars())],
    );

    let bars = client.get_latest_stock_bars(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests.len(), 2);
    assert_eq!(bars["AAPL"].close, 172.79);
}

#[test]
fn get_latest_stock_bars_retries_when_rate_limit_reset_has_passed() {
    let (client, server) = server(
        retries(2),
        vec![
            reply_with_headers(
                "429 Too Many Requests",
                json!({}),
                "X-RateLimit-Reset: 1\r\n",
            ),
            reply("200 OK", latest_bars()),
        ],
    );

    let bars = client.get_latest_stock_bars(&["AAPL"]).unwrap();
    let requests = server.join().unwrap();

    assert_eq!(requests.len(), 2);
    assert_eq!(bars["AAPL"].close, 172.79);
}

#[test]
fn get_latest_stock_bars_gives_up_when_rate_limit_reset_exceeds_max_delay() {
    let (client, server) = server(
        retries(3),
        vec![reply_with_headers(
            "429 Too Many Requests",
            json!({}),
            "X-RateLimit-Reset: 99999999999\r\n",
        )],
    );

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 429 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_latest_stock_bars_does_not_retry_when_status_is_400() {
    let (client, server) = server(retries(3), vec![reply("400 Bad Request", json!({}))]);

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 400 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_latest_stock_bars_does_not_retry_when_status_is_403() {
    let (client, server) = server(retries(3), vec![reply("403 Forbidden", json!({}))]);

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 403 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_latest_stock_bars_does_not_retry_when_json_is_malformed() {
    let (client, server) = server(retries(3), vec![reply("200 OK", json!({"bars": 1}))]);

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::InvalidResponse);
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_latest_stock_bars_returns_last_status_when_retries_are_exhausted() {
    let (client, server) = server(
        retries(3),
        vec![
            reply("503 Service Unavailable", json!({})),
            reply("429 Too Many Requests", json!({})),
            reply("500 Internal Server Error", json!({})),
        ],
    );

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 500 });
    assert_eq!(requests.len(), 3);
}

#[test]
fn get_latest_stock_bars_returns_transport_error_when_retries_are_exhausted() {
    let (client, server) = server(retries(2), vec![Reply::Close, Reply::Close]);

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::Transport { timed_out: false });
    assert_eq!(requests.len(), 2);
}

#[test]
fn get_latest_stock_bars_sends_one_request_when_policy_is_none() {
    let config = DataClientConfig {
        feed: None,
        retry: RetryPolicy::none(),
    };
    let (client, server) = server(config, vec![reply("503 Service Unavailable", json!({}))]);

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 503 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_latest_stock_bars_sends_one_request_when_max_attempts_is_zero() {
    let (client, server) = server(
        retries(0),
        vec![reply("503 Service Unavailable", json!({}))],
    );

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    let requests = server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 503 });
    assert_eq!(requests.len(), 1);
}

#[test]
fn get_latest_stock_bars_reports_status_without_body_when_retries_are_exhausted() {
    let (client, server) = server(
        retries(2),
        vec![
            reply(
                "500 Internal Server Error",
                json!({"message": "sensitive detail"}),
            ),
            reply(
                "500 Internal Server Error",
                json!({"message": "sensitive detail"}),
            ),
        ],
    );

    let error = client.get_latest_stock_bars(&["AAPL"]).unwrap_err();
    server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 500 });
    assert!(!format!("{error:?} {error}").contains("sensitive detail"));
}
