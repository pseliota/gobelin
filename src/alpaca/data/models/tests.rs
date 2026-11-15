use std::collections::HashMap;

use super::*;
use crate::transport::testing::query_pairs;
use serde_json::{Value, json};

const LATEST_BARS: &str = r#"{"bars":{"AAPL":{"c":172.79,"h":172.81,"l":172.78,"n":20,"o":172.81,"t":"2022-08-17T08:58:00Z","v":1002,"vw":172.791417},"TSLA":{"c":914.3,"h":914.3,"l":914.3,"n":20,"o":914.3,"t":"2022-08-17T08:57:00Z","v":751,"vw":914.294634}}}"#;
const LATEST_TRADES: &str = r#"{"trades":{"AAPL":{"c":["@","F","T"],"i":826,"p":172.78,"s":100,"t":"2022-08-17T09:50:43.361102308Z","x":"Q","z":"C"}}}"#;
const SNAPSHOTS: &str = r#"{"AAPL":{"dailyBar":{"c":173.03,"h":173.71,"l":171.6618,"n":515139,"o":172.62,"t":"2022-08-16T04:00:00Z","v":56457696,"vw":172.743391},"latestQuote":{"ap":172.7,"as":5,"ax":"Q","bp":172.6,"bs":2,"bx":"Q","c":["R"],"t":"2022-08-17T10:18:27.052763263Z","z":"C"},"latestTrade":{"c":["@","T"],"i":1011,"p":172.61,"s":160,"t":"2022-08-17T10:18:24.114694956Z","x":"Q","z":"C"},"minuteBar":{"c":172.69,"h":172.69,"l":172.69,"n":3,"o":172.69,"t":"2022-08-17T10:16:00Z","v":106,"vw":172.688113},"prevDailyBar":{"c":173.19,"h":173.39,"l":171.345,"n":501626,"o":171.5,"t":"2022-08-15T04:00:00Z","v":54091719,"vw":172.625371}}}"#;

fn member(document: &str, path: &[&str]) -> Value {
    let mut value: Value = serde_json::from_str(document).unwrap();
    for key in path {
        value = value[*key].take();
    }
    value
}

fn bars_request() -> BarsRequest {
    BarsRequest {
        symbols: vec!["AAPL".into()],
        timeframe: "1Day".into(),
        ..Default::default()
    }
}

#[test]
fn deserialize_maps_bar_fields_when_wire_keys_are_abbreviated() {
    let response = member(LATEST_BARS, &["bars", "AAPL"]);

    let bar: Bar = serde_json::from_value(response).unwrap();

    assert_eq!(
        bar,
        Bar {
            timestamp: "2022-08-17T08:58:00Z".into(),
            open: 172.81,
            high: 172.81,
            low: 172.78,
            close: 172.79,
            volume: 1002,
            trade_count: 20,
            vwap: 172.791417,
        }
    );
}

#[test]
fn deserialize_keys_bars_by_symbol_when_response_has_several_symbols() {
    let response = member(LATEST_BARS, &["bars"]);

    let bars: HashMap<String, Bar> = serde_json::from_value(response).unwrap();

    assert_eq!(bars.len(), 2);
    assert_eq!(bars["TSLA"].close, 914.3);
}

#[test]
fn deserialize_rejects_bar_when_close_is_missing() {
    let mut response = member(LATEST_BARS, &["bars", "AAPL"]);
    response.as_object_mut().unwrap().remove("c");

    let result = serde_json::from_value::<Bar>(response);

    assert!(result.is_err());
}

#[test]
fn deserialize_ignores_bar_fields_when_they_are_unmodeled() {
    let mut response = member(LATEST_BARS, &["bars", "AAPL"]);
    response["future_field"] = json!(1);

    let result = serde_json::from_value::<Bar>(response);

    assert!(result.is_ok());
}

#[test]
fn deserialize_maps_trade_fields_when_wire_keys_are_abbreviated() {
    let response = member(LATEST_TRADES, &["trades", "AAPL"]);

    let trade: Trade = serde_json::from_value(response).unwrap();

    assert_eq!(
        trade,
        Trade {
            timestamp: "2022-08-17T09:50:43.361102308Z".into(),
            exchange: "Q".into(),
            price: 172.78,
            size: 100,
            conditions: vec!["@".into(), "F".into(), "T".into()],
            id: 826,
            tape: "C".into(),
            update: None,
        }
    );
}

#[test]
fn deserialize_preserves_trade_update_when_trade_was_corrected() {
    let mut response = member(LATEST_TRADES, &["trades", "AAPL"]);
    response["u"] = json!("corrected");

    let trade: Trade = serde_json::from_value(response).unwrap();

    assert_eq!(trade.update.as_deref(), Some("corrected"));
}

#[test]
fn deserialize_leaves_trade_conditions_empty_when_field_is_absent() {
    let mut response = member(LATEST_TRADES, &["trades", "AAPL"]);
    response.as_object_mut().unwrap().remove("c");

    let trade: Trade = serde_json::from_value(response).unwrap();

    assert!(trade.conditions.is_empty());
}

#[test]
fn deserialize_maps_quote_fields_when_wire_keys_are_abbreviated() {
    let response = member(SNAPSHOTS, &["AAPL", "latestQuote"]);

    let quote: Quote = serde_json::from_value(response).unwrap();

    assert_eq!(
        quote,
        Quote {
            timestamp: "2022-08-17T10:18:27.052763263Z".into(),
            ask_exchange: "Q".into(),
            ask_price: 172.7,
            ask_size: 5,
            bid_exchange: "Q".into(),
            bid_price: 172.6,
            bid_size: 2,
            conditions: vec!["R".into()],
            tape: "C".into(),
        }
    );
}

#[test]
fn deserialize_leaves_quote_conditions_empty_when_field_is_absent() {
    let mut response = member(SNAPSHOTS, &["AAPL", "latestQuote"]);
    response.as_object_mut().unwrap().remove("c");

    let quote: Quote = serde_json::from_value(response).unwrap();

    assert!(quote.conditions.is_empty());
}

#[test]
fn deserialize_maps_snapshot_members_when_wire_keys_are_camel_case() {
    let snapshots: HashMap<String, Snapshot> = serde_json::from_str(SNAPSHOTS).unwrap();

    let snapshot = &snapshots["AAPL"];

    assert_eq!(snapshot.latest_trade.as_ref().unwrap().id, 1011);
    assert_eq!(snapshot.latest_quote.as_ref().unwrap().ask_price, 172.7);
    assert_eq!(snapshot.minute_bar.as_ref().unwrap().volume, 106);
    assert_eq!(snapshot.daily_bar.as_ref().unwrap().volume, 56457696);
    assert_eq!(snapshot.prev_daily_bar.as_ref().unwrap().close, 173.19);
}

#[test]
fn deserialize_leaves_snapshot_members_unknown_when_they_are_absent() {
    let response = json!({});

    let snapshot: Snapshot = serde_json::from_value(response).unwrap();

    assert_eq!(
        snapshot,
        Snapshot {
            latest_trade: None,
            latest_quote: None,
            minute_bar: None,
            daily_bar: None,
            prev_daily_bar: None,
        }
    );
}

#[test]
fn deserialize_leaves_snapshot_members_unknown_when_they_are_null() {
    let mut response = member(SNAPSHOTS, &["AAPL"]);
    response["latestTrade"] = json!(null);
    response["minuteBar"] = json!(null);

    let snapshot: Snapshot = serde_json::from_value(response).unwrap();

    assert_eq!(snapshot.latest_trade, None);
    assert_eq!(snapshot.minute_bar, None);
    assert!(snapshot.daily_bar.is_some());
}

#[test]
fn deserialize_parses_bars_page_when_token_is_present() {
    let bar = member(LATEST_BARS, &["bars", "AAPL"]);
    let response = json!({
        "bars": {"AAPL": [bar.clone(), bar]},
        "next_page_token": "abc",
        "currency": "USD"
    });

    let page: BarsPage = serde_json::from_value(response).unwrap();

    assert_eq!(page.bars["AAPL"].len(), 2);
    assert_eq!(page.bars["AAPL"][0].close, 172.79);
    assert_eq!(page.next_page_token.as_deref(), Some("abc"));
    assert_eq!(page.currency.as_deref(), Some("USD"));
}

#[test]
fn deserialize_leaves_page_token_unknown_when_value_is_null() {
    let response = json!({"bars": {"AAPL": []}, "next_page_token": null});

    let page: BarsPage = serde_json::from_value(response).unwrap();

    assert_eq!(page.next_page_token, None);
    assert_eq!(page.currency, None);
}

#[test]
fn deserialize_leaves_bars_empty_when_key_is_missing() {
    let response = json!({"next_page_token": null});

    let page: BarsPage = serde_json::from_value(response).unwrap();

    assert!(page.bars.is_empty());
}

#[test]
fn deserialize_leaves_bars_empty_when_value_is_null() {
    let response = json!({"bars": null, "next_page_token": null});

    let page: BarsPage = serde_json::from_value(response).unwrap();

    assert!(page.bars.is_empty());
}

#[test]
fn deserialize_leaves_bars_empty_when_value_is_empty_object() {
    let response = json!({"bars": {}, "next_page_token": null});

    let page: BarsPage = serde_json::from_value(response).unwrap();

    assert!(page.bars.is_empty());
}

#[test]
fn serialize_joins_symbols_with_commas_when_request_has_several() {
    let mut request = bars_request();
    request.symbols = vec!["AAPL".into(), "TSLA".into(), "BRK.B".into()];

    let pairs = query_pairs(&request);

    assert_eq!(
        pairs,
        vec![
            ("symbols".to_string(), "AAPL,TSLA,BRK.B".to_string()),
            ("timeframe".to_string(), "1Day".to_string()),
        ]
    );
}

#[test]
fn serialize_joins_adjustments_with_commas_when_request_has_several() {
    let mut request = bars_request();
    request.adjustment = vec![Adjustment::Split, Adjustment::Dividend];

    let pairs = query_pairs(&request);

    assert_eq!(
        pairs
            .iter()
            .filter(|(key, _)| key == "adjustment")
            .collect::<Vec<_>>(),
        vec![&("adjustment".to_string(), "split,dividend".to_string())]
    );
}

#[test]
fn serialize_hyphenates_adjustment_when_value_is_spin_off() {
    let mut request = bars_request();
    request.adjustment = vec![Adjustment::SpinOff];

    let pairs: HashMap<_, _> = query_pairs(&request).into_iter().collect();

    assert_eq!(pairs["adjustment"], "spin-off");
}

#[test]
fn serialize_names_adjustments_when_each_variant_is_encoded() {
    let adjustments = [
        Adjustment::Raw,
        Adjustment::Split,
        Adjustment::Dividend,
        Adjustment::SpinOff,
        Adjustment::All,
    ];

    let names = serde_json::to_value(adjustments).unwrap();

    assert_eq!(
        names,
        json!(["raw", "split", "dividend", "spin-off", "all"])
    );
}

#[test]
fn serialize_omits_options_when_bars_request_sets_only_required_fields() {
    let request = bars_request();

    let pairs: HashMap<_, _> = query_pairs(&request).into_iter().collect();

    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs["symbols"], "AAPL");
    assert_eq!(pairs["timeframe"], "1Day");
}

#[test]
fn serialize_encodes_options_when_bars_request_sets_all_fields() {
    let request = BarsRequest {
        symbols: vec!["AAPL".into()],
        timeframe: "5Min".into(),
        start: Some("2024-01-03T00:00:00Z".into()),
        end: Some("2024-01-04T00:00:00Z".into()),
        limit: Some(1000),
        adjustment: vec![Adjustment::All],
        asof: Some("2024-01-05".into()),
        currency: Some("EUR".into()),
        sort: Some(SortDirection::Desc),
    };

    let pairs: HashMap<_, _> = query_pairs(&request).into_iter().collect();

    assert_eq!(pairs.len(), 9);
    assert_eq!(pairs["timeframe"], "5Min");
    assert_eq!(pairs["start"], "2024-01-03T00:00:00Z");
    assert_eq!(pairs["end"], "2024-01-04T00:00:00Z");
    assert_eq!(pairs["limit"], "1000");
    assert_eq!(pairs["adjustment"], "all");
    assert_eq!(pairs["asof"], "2024-01-05");
    assert_eq!(pairs["currency"], "EUR");
    assert_eq!(pairs["sort"], "desc");
}

#[test]
fn serialize_names_feeds_when_each_variant_is_encoded() {
    let feeds = [
        Feed::Iex,
        Feed::Sip,
        Feed::DelayedSip,
        Feed::Otc,
        Feed::Boats,
        Feed::Overnight,
    ];

    let names = serde_json::to_value(feeds).unwrap();

    assert_eq!(
        names,
        json!(["iex", "sip", "delayed_sip", "otc", "boats", "overnight"])
    );
}
