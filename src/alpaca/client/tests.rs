use super::super::models::{OrderSide, OrderType, QueryOrderStatus, TimeInForce};
use super::*;
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    net::TcpListener,
    thread::{self, JoinHandle},
    time::Instant,
};

struct RecordedRequest {
    method: String,
    target: String,
    headers: std::collections::HashMap<String, String>,
    body: Vec<u8>,
}

// One bounded loopback exchange; no credentials, proxies, or external services.
fn server(status: &str, body: Value) -> (AlpacaClient, JoinHandle<RecordedRequest>) {
    raw_server(status, &body.to_string(), "")
}

fn raw_server(
    status: &str,
    body: &str,
    extra_headers: &str,
) -> (AlpacaClient, JoinHandle<RecordedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let response = if status.is_empty() {
        String::new()
    } else {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra_headers}\r\n{body}",
            body.len()
        )
    };
    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "client never connected");
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept failed: {error}"),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
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
        let headers: std::collections::HashMap<_, _> = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| (key.to_ascii_lowercase(), value.trim().to_owned()))
            .collect();
        let length = headers
            .get("content-length")
            .map(|n| n.parse::<usize>().unwrap())
            .unwrap_or(0);
        while bytes.len() < header_end + length {
            let mut buffer = [0; 4096];
            let count = stream.read(&mut buffer).unwrap();
            assert!(count > 0, "incomplete body");
            bytes.extend_from_slice(&buffer[..count]);
        }
        stream.write_all(response.as_bytes()).unwrap();
        RecordedRequest {
            method,
            target,
            headers,
            body: bytes[header_end..header_end + length].to_vec(),
        }
    });
    let client = AlpacaClient::build(
        "test-key",
        "test-secret",
        Url::parse(&format!("http://{address}/v2/")).unwrap(),
    )
    .unwrap();
    (client, handle)
}

fn order_response() -> Value {
    json!({
        "id": "order-1", "client_order_id": "client-1",
        "created_at": "2025-01-02T12:00:00Z", "updated_at": "2025-01-02T12:00:00Z",
        "submitted_at": "2025-01-02T12:00:00Z", "time_in_force": "day",
        "status": "accepted", "extended_hours": false
    })
}

#[test]
fn new_uses_paper_endpoint_when_credentials_are_valid() {
    let client = AlpacaClient::new("test-key", "test-secret").unwrap();

    assert_eq!(client.base_url.as_str(), PAPER_URL);
}

#[test]
fn new_rejects_credentials_when_secret_contains_newline() {
    let result = AlpacaClient::new("test-key", "secret\nvalue");

    assert!(matches!(result, Err(ClientError::InvalidCredentials)));
}

#[test]
fn new_rejects_credentials_when_key_is_blank() {
    let result = AlpacaClient::new(" ", "test-secret");

    assert!(matches!(result, Err(ClientError::InvalidCredentials)));
}

#[test]
fn credential_header_redacts_value_when_formatted_for_debug() {
    let header = credential_header("test-secret").unwrap();

    let debug = format!("{header:?}");

    assert!(header.is_sensitive());
    assert!(!debug.contains("test-secret"));
}

#[test]
fn get_account_authenticates_request_when_fetching_account() {
    let (client, server) = server("200 OK", json!({"id": "account-1", "status": "ACTIVE"}));

    let account = client.get_account().unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.method, "GET");
    assert_eq!(request.target, "/v2/account");
    assert_eq!(request.headers["apca-api-key-id"], "test-key");
    assert_eq!(request.headers["apca-api-secret-key"], "test-secret");
    assert_eq!(account.id, "account-1");
}

#[test]
fn list_assets_encodes_filters_when_query_is_supplied() {
    let (client, server) = server("200 OK", json!([]));
    let filters = ListAssetsRequest {
        asset_class: Some("us_equity".into()),
        ..Default::default()
    };

    let assets = client.list_assets(&filters).unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.method, "GET");
    assert_eq!(request.target, "/v2/assets?asset_class=us_equity");
    assert!(assets.is_empty());
}

#[test]
fn get_asset_encodes_identifier_when_symbol_contains_reserved_characters() {
    let (client, server) = server("404 Not Found", json!({}));

    let result = client.get_asset("BTC/USD?x=1#fragment");
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/assets/BTC%2FUSD%3Fx=1%23fragment");
    assert_eq!(result.unwrap_err(), ClientError::HttpStatus { status: 404 });
}

#[test]
fn get_position_uses_position_path_when_identifier_is_supplied() {
    let (client, server) = server(
        "200 OK",
        json!({
            "asset_id": "asset-1", "symbol": "AAPL", "exchange": "NASDAQ",
            "asset_class": "us_equity", "qty": "2", "side": "long",
            "avg_entry_price": "150", "cost_basis": "300"
        }),
    );

    let position = client.get_position("AAPL").unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/positions/AAPL");
    assert_eq!(position.qty, "2");
}

#[test]
fn list_positions_returns_empty_list_when_no_positions_exist() {
    let (client, server) = server("200 OK", json!([]));

    let positions = client.list_positions().unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/positions");
    assert!(positions.is_empty());
}

#[test]
fn list_orders_sends_filters_when_requesting_closed_orders() {
    let (client, server) = server("200 OK", json!([order_response()]));
    let filters = ListOrdersRequest {
        status: Some(QueryOrderStatus::Closed),
        ..Default::default()
    };

    let orders = client.list_orders(&filters).unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/orders?status=closed");
    assert_eq!(orders[0].id, "order-1");
}

#[test]
fn get_order_requests_nested_legs_when_enabled() {
    let (client, server) = server("200 OK", order_response());

    let order = client.get_order("order-1", true).unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/orders/order-1?nested=true");
    assert_eq!(order.id, "order-1");
}

#[test]
fn get_order_by_client_id_encodes_query_when_id_contains_ampersand() {
    let (client, server) = server("200 OK", order_response());

    client.get_order_by_client_id("client&1").unwrap();
    let request = server.join().unwrap();

    assert_eq!(
        request.target,
        "/v2/orders:by_client_order_id?client_order_id=client%261"
    );
}

#[test]
fn create_order_sends_json_when_submitting_market_order() {
    let (client, server) = server("200 OK", order_response());
    let order = CreateOrderRequest {
        order_type: OrderType::Market,
        time_in_force: TimeInForce::Day,
        symbol: Some("AAPL".into()),
        qty: Some("2".into()),
        side: Some(OrderSide::Buy),
        client_order_id: Some("client-1".into()),
        notional: None,
        limit_price: None,
        stop_price: None,
        trail_price: None,
        trail_percent: None,
        extended_hours: None,
        order_class: None,
        take_profit: None,
        stop_loss: None,
        position_intent: None,
        legs: None,
        advanced_instructions: None,
    };

    let result = client.create_order(&order).unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.method, "POST");
    assert_eq!(request.target, "/v2/orders");
    assert_eq!(request.headers["content-type"], "application/json");
    assert_eq!(
        serde_json::from_slice::<Value>(&request.body).unwrap(),
        json!({
            "type": "market", "time_in_force": "day", "symbol": "AAPL",
            "qty": "2", "side": "buy", "client_order_id": "client-1"
        })
    );
    assert_eq!(result.id, "order-1");
}

#[test]
fn replace_order_sends_patch_when_price_changes() {
    let (client, server) = server("200 OK", order_response());
    let replacement = ReplaceOrderRequest {
        limit_price: Some("155.00".into()),
        ..Default::default()
    };

    client.replace_order("order-1", &replacement).unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.method, "PATCH");
    assert_eq!(request.target, "/v2/orders/order-1");
    assert_eq!(
        serde_json::from_slice::<Value>(&request.body).unwrap(),
        json!({"limit_price": "155.00"})
    );
}

#[test]
fn cancel_order_accepts_empty_body_when_status_is_204() {
    let (client, server) = raw_server("204 No Content", "", "");

    let result = client.cancel_order("order-1");
    let request = server.join().unwrap();

    assert_eq!(result, Ok(()));
    assert_eq!(request.method, "DELETE");
    assert_eq!(request.target, "/v2/orders/order-1");
}

#[test]
fn cancel_all_orders_preserves_failures_when_results_are_mixed() {
    let (client, server) = server(
        "207 Multi-Status",
        json!([
            {"id": "order-1", "status": 200}, {"id": "order-2", "status": 500}
        ]),
    );

    let results = client.cancel_all_orders().unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.method, "DELETE");
    assert_eq!(request.target, "/v2/orders");
    assert_eq!(results[1].status, 500);
}

#[test]
fn get_clock_decodes_response_when_market_is_closed() {
    let (client, server) = server(
        "200 OK",
        json!({
            "timestamp": "2025-01-02T08:00:00-05:00", "is_open": false,
            "next_open": "2025-01-02T09:30:00-05:00", "next_close": "2025-01-02T16:00:00-05:00"
        }),
    );

    let clock = client.get_clock().unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/clock");
    assert!(!clock.is_open);
}

#[test]
fn get_calendar_encodes_dates_when_range_is_supplied() {
    let (client, server) = server("200 OK", json!([]));
    let filters = CalendarRequest {
        start: Some("2025-01-02".into()),
        ..Default::default()
    };

    let days = client.get_calendar(&filters).unwrap();
    let request = server.join().unwrap();

    assert_eq!(request.target, "/v2/calendar?start=2025-01-02");
    assert!(days.is_empty());
}

#[test]
fn get_account_reports_status_without_body_when_authentication_fails() {
    let (client, server) = server(
        "401 Unauthorized",
        json!({"message": "sensitive account information"}),
    );

    let error = client.get_account().unwrap_err();
    server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 401 });
    assert!(!format!("{error:?} {error}").contains("sensitive account information"));
}

#[test]
fn get_account_rejects_redirect_when_server_returns_location() {
    let (client, server) = raw_server("302 Found", "", "Location: https://example.invalid/\r\n");

    let error = client.get_account().unwrap_err();
    server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 302 });
}

#[test]
fn get_account_reports_invalid_response_when_json_is_malformed() {
    let (client, server) = raw_server("200 OK", "not json", "");

    let error = client.get_account().unwrap_err();
    server.join().unwrap();

    assert_eq!(error, ClientError::InvalidResponse);
}

#[test]
fn get_order_rejects_identifier_when_path_is_dot_segment() {
    let client = AlpacaClient::new("test-key", "test-secret").unwrap();

    let error = client.get_order("..", false).unwrap_err();

    assert_eq!(error, ClientError::InvalidPath);
}

#[test]
fn get_account_reports_transport_failure_when_server_closes_without_response() {
    let (client, server) = raw_server("", "", "");

    let error = client.get_account().unwrap_err();
    server.join().unwrap();

    assert_eq!(error, ClientError::Transport { timed_out: false });
}

#[test]
fn cancel_order_reports_rejection_when_order_is_not_cancelable() {
    let (client, server) = server("422 Unprocessable Entity", json!({}));

    let error = client.cancel_order("order-1").unwrap_err();
    server.join().unwrap();

    assert_eq!(error, ClientError::HttpStatus { status: 422 });
}
