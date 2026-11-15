use super::*;
use serde_json::{Value, json};

#[test]
fn deserialize_preserves_account_precision_when_decimal_has_many_digits() {
    let response = json!({
        "id": "account-id", "status": "ACTIVE",
        "last_equity": "122011.09751111286868"
    });

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(
        account.last_equity.as_deref(),
        Some("122011.09751111286868")
    );
}

#[test]
fn deserialize_preserves_account_status_when_value_is_unknown() {
    let response = json!({"id": "account-id", "status": "FUTURE_STATUS"});

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(account.status, "FUTURE_STATUS");
}

#[test]
fn deserialize_ignores_account_fields_when_they_are_unmodeled() {
    let response = json!({
        "id": "account-id", "status": "ACTIVE", "admin_configurations": {}
    });

    let result = serde_json::from_value::<Account>(response);

    assert!(result.is_ok());
}

#[test]
fn deserialize_leaves_account_number_unknown_when_value_is_null() {
    let response = json!({"id": "account-id", "status": "ACTIVE", "account_number": null});

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(account.account_number, None);
}

#[test]
fn deserialize_leaves_daytrade_count_unknown_when_legacy_field_is_absent() {
    let response = json!({"id": "account-id", "status": "ACTIVE"});

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(account.daytrade_count, None);
}

#[test]
fn deserialize_leaves_cash_unknown_when_balance_is_absent() {
    let response = json!({"id": "account-id", "status": "ONBOARDING"});

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(account.cash, None);
}

#[test]
fn deserialize_leaves_trading_permission_unknown_when_flag_is_absent() {
    let response = json!({"id": "account-id", "status": "ONBOARDING"});

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(account.trading_blocked, None);
}

#[test]
fn deserialize_preserves_trading_permission_when_flag_is_false() {
    let response = json!({"id": "account-id", "status": "ACTIVE", "trading_blocked": false});

    let account: Account = serde_json::from_value(response).unwrap();

    assert_eq!(account.trading_blocked, Some(false));
}

#[test]
fn deserialize_rejects_account_when_id_is_missing() {
    let response = json!({"status": "ACTIVE"});

    let result = serde_json::from_value::<Account>(response);

    assert!(result.is_err());
}

#[test]
fn deserialize_rejects_cash_when_value_is_number_instead_of_string() {
    let response = json!({"id": "account-id", "status": "ACTIVE", "cash": 1.25});

    let result = serde_json::from_value::<Account>(response);

    assert!(result.is_err());
}

fn asset_response() -> Value {
    json!({
        "id": "asset-id", "class": "crypto", "exchange": "CRYPTO",
        "symbol": "BTC/USD", "name": "Bitcoin", "status": "active",
        "tradable": true, "marginable": false, "shortable": false,
        "fractionable": true
    })
}

#[test]
fn deserialize_maps_asset_class_when_wire_field_is_class() {
    let mut response = asset_response();
    response["class"] = json!("crypto");

    let asset: Asset = serde_json::from_value(response).unwrap();

    assert_eq!(asset.asset_class, "crypto");
}

#[test]
fn deserialize_preserves_minimum_size_when_crypto_quantity_is_fractional() {
    let mut response = asset_response();
    response["min_order_size"] = json!("0.000000001");

    let asset: Asset = serde_json::from_value(response).unwrap();

    assert_eq!(asset.min_order_size.as_deref(), Some("0.000000001"));
}

#[test]
fn deserialize_leaves_borrow_status_unknown_when_field_is_absent() {
    let response = asset_response();

    let asset: Asset = serde_json::from_value(response).unwrap();

    assert_eq!(asset.borrow_status, None);
}

#[test]
fn deserialize_leaves_borrow_flag_unknown_when_legacy_field_is_null() {
    let mut response = asset_response();
    response["easy_to_borrow"] = json!(null);

    let asset: Asset = serde_json::from_value(response).unwrap();

    assert_eq!(asset.easy_to_borrow, None);
}

#[test]
fn deserialize_preserves_asset_attributes_when_value_is_unknown() {
    let mut response = asset_response();
    response["attributes"] = json!(["future_attribute"]);

    let asset: Asset = serde_json::from_value(response).unwrap();

    assert_eq!(asset.attributes.unwrap(), vec!["future_attribute"]);
}

fn position_response() -> Value {
    json!({
        "asset_id": "asset-id", "symbol": "AAPL", "exchange": "NASDAQ",
        "asset_class": "us_equity", "avg_entry_price": "174.78",
        "qty": "2", "side": "long", "cost_basis": "349.56"
    })
}

#[test]
fn deserialize_preserves_signed_quantity_when_position_is_short() {
    let mut response = position_response();
    response["qty"] = json!("-2");
    response["side"] = json!("short");
    response["cost_basis"] = json!("-349.56");

    let position: Position = serde_json::from_value(response).unwrap();

    assert_eq!(position.qty, "-2");
}

#[test]
fn deserialize_leaves_market_value_unknown_when_valuation_is_null() {
    let mut response = position_response();
    response["market_value"] = json!(null);

    let position: Position = serde_json::from_value(response).unwrap();

    assert_eq!(position.market_value, None);
}

#[test]
fn deserialize_leaves_current_price_unknown_when_valuation_is_absent() {
    let response = position_response();

    let position: Position = serde_json::from_value(response).unwrap();

    assert_eq!(position.current_price, None);
}

#[test]
fn deserialize_parses_usd_values_when_position_contains_currency_conversion() {
    let mut response = position_response();
    response["usd"] = json!({"avg_entry_price": "174.78", "cost_basis": "349.56"});

    let position: Position = serde_json::from_value(response).unwrap();

    assert_eq!(position.usd.unwrap().cost_basis, "349.56");
}

fn order_response() -> Value {
    json!({
        "id": "order-id", "client_order_id": "client-id",
        "created_at": "2023-12-12T22:31:24Z",
        "updated_at": "2023-12-12T22:31:24Z",
        "submitted_at": "2023-12-12T22:31:24Z",
        "time_in_force": "gtc", "status": "accepted", "extended_hours": false
    })
}

#[test]
fn deserialize_uses_current_order_type_when_deprecated_field_is_also_present() {
    let mut response = order_response();
    response["type"] = json!("limit");
    response["order_type"] = json!("market");

    let order: Order = serde_json::from_value(response).unwrap();

    assert_eq!(order.order_type.as_deref(), Some("limit"));
}

#[test]
fn deserialize_preserves_order_class_when_simple_order_uses_empty_string() {
    let mut response = order_response();
    response["order_class"] = json!("");

    let order: Order = serde_json::from_value(response).unwrap();

    assert_eq!(order.order_class.as_deref(), Some(""));
}

#[test]
fn deserialize_preserves_timestamp_when_order_has_nanosecond_precision() {
    let mut response = order_response();
    response["created_at"] = json!("2023-12-12T22:31:24.668464435Z");

    let order: Order = serde_json::from_value(response).unwrap();

    assert_eq!(order.created_at, "2023-12-12T22:31:24.668464435Z");
}

#[test]
fn deserialize_accepts_optional_order_fields_when_values_are_null() {
    let mut response = order_response();
    response["notional"] = json!(null);
    response["filled_avg_price"] = json!(null);
    response["legs"] = json!(null);

    let order: Order = serde_json::from_value(response).unwrap();

    assert_eq!(order.notional, None);
    assert_eq!(order.filled_avg_price, None);
    assert!(order.legs.is_none());
}

#[test]
fn deserialize_preserves_empty_identifiers_when_order_is_multileg() {
    let mut response = order_response();
    response["order_class"] = json!("mleg");
    response["symbol"] = json!("");
    response["side"] = json!("");
    response["asset_class"] = json!("");

    let order: Order = serde_json::from_value(response).unwrap();

    assert_eq!(order.symbol.as_deref(), Some(""));
    assert_eq!(order.side.as_deref(), Some(""));
    assert_eq!(order.asset_class.as_deref(), Some(""));
}

#[test]
fn deserialize_preserves_order_status_when_value_is_unknown() {
    let mut response = order_response();
    response["status"] = json!("future_status");

    let order: Order = serde_json::from_value(response).unwrap();

    assert_eq!(order.status, "future_status");
}

#[test]
fn deserialize_parses_nested_leg_when_multileg_response_contains_ratio() {
    let mut response = order_response();
    let mut leg = order_response();
    leg["type"] = json!("");
    leg["ratio_qty"] = json!("3");
    response["order_class"] = json!("mleg");
    response["legs"] = json!([leg]);

    let order: Order = serde_json::from_value(response).unwrap();

    let legs = order.legs.unwrap();
    assert_eq!(legs.len(), 1);
    assert_eq!(legs[0].order_type.as_deref(), Some(""));
    assert_eq!(legs[0].ratio_qty.as_deref(), Some("3"));
}

fn create_order() -> CreateOrderRequest {
    CreateOrderRequest {
        order_type: OrderType::Limit,
        time_in_force: TimeInForce::Gtc,
        symbol: Some("AAPL".into()),
        qty: Some("2".into()),
        notional: None,
        side: Some(OrderSide::Buy),
        limit_price: Some("150".into()),
        stop_price: None,
        trail_price: None,
        trail_percent: None,
        extended_hours: None,
        client_order_id: None,
        order_class: None,
        take_profit: None,
        stop_loss: None,
        position_intent: None,
        legs: None,
        advanced_instructions: None,
    }
}

#[test]
fn serialize_matches_wire_contract_when_creating_simple_limit_order() {
    let request = create_order();
    let expected = json!({
        "symbol": "AAPL", "qty": "2", "side": "buy",
        "type": "limit", "time_in_force": "gtc", "limit_price": "150"
    });

    let body = serde_json::to_value(request).unwrap();

    assert_eq!(body, expected);
}

#[test]
fn serialize_preserves_false_when_extended_hours_is_explicitly_disabled() {
    let mut request = create_order();
    request.extended_hours = Some(false);

    let body = serde_json::to_value(request).unwrap();

    assert_eq!(body["extended_hours"], false);
}

#[test]
fn serialize_nests_exit_prices_when_order_is_bracket() {
    let mut request = create_order();
    request.order_class = Some(OrderClass::Bracket);
    request.take_profit = Some(TakeProfit {
        limit_price: "170.00".into(),
    });
    request.stop_loss = Some(StopLoss {
        stop_price: "140.00".into(),
        limit_price: None,
    });

    let body = serde_json::to_value(request).unwrap();

    assert_eq!(body["order_class"], "bracket");
    assert_eq!(body["take_profit"], json!({"limit_price": "170.00"}));
    assert_eq!(body["stop_loss"], json!({"stop_price": "140.00"}));
}

#[test]
fn serialize_omits_top_level_identity_when_multileg_order_uses_legs() {
    let mut request = create_order();
    request.order_class = Some(OrderClass::Mleg);
    request.symbol = None;
    request.side = None;
    request.time_in_force = TimeInForce::Day;
    request.legs = Some(vec![
        OrderLegRequest {
            symbol: "AAPL241213C00250000".into(),
            ratio_qty: "3".into(),
            side: Some(OrderSide::Buy),
            position_intent: Some(PositionIntent::BuyToOpen),
        },
        OrderLegRequest {
            symbol: "AAPL241213C00260000".into(),
            ratio_qty: "1".into(),
            side: Some(OrderSide::Sell),
            position_intent: Some(PositionIntent::SellToOpen),
        },
    ]);

    let body = serde_json::to_value(request).unwrap();

    assert!(body.get("symbol").is_none());
    assert!(body.get("side").is_none());
    assert_eq!(body["legs"][0]["symbol"], "AAPL241213C00250000");
    assert_eq!(body["legs"][1]["side"], "sell");
}

#[test]
fn serialize_preserves_leg_fields_when_position_intent_is_supplied() {
    let leg = OrderLegRequest {
        symbol: "AAPL241213C00260000".into(),
        ratio_qty: "3".into(),
        side: Some(OrderSide::Sell),
        position_intent: Some(PositionIntent::SellToOpen),
    };

    let body = serde_json::to_value(leg).unwrap();

    assert_eq!(
        body,
        json!({
            "symbol": "AAPL241213C00260000", "ratio_qty": "3",
            "side": "sell", "position_intent": "sell_to_open"
        })
    );
}

#[test]
fn serialize_sends_only_price_when_replacement_changes_limit() {
    let request = ReplaceOrderRequest {
        limit_price: Some("155.000000001".into()),
        ..Default::default()
    };

    let body = serde_json::to_value(request).unwrap();

    assert_eq!(body, json!({"limit_price": "155.000000001"}));
}

#[test]
fn serialize_omits_changes_when_replacement_is_default() {
    let request = ReplaceOrderRequest::default();

    let body = serde_json::to_value(request).unwrap();

    assert_eq!(body, json!({}));
}

#[test]
fn serialize_encodes_order_query_when_filters_are_supplied() {
    let filters = ListOrdersRequest {
        status: Some(QueryOrderStatus::All),
        limit: Some(100),
        direction: Some(SortDirection::Asc),
        nested: Some(false),
        symbols: Some("AAPL,MSFT".into()),
        ..Default::default()
    };
    let client = reqwest::blocking::Client::new();

    let request = client
        .get("http://localhost/v2/orders")
        .query(&filters)
        .build()
        .unwrap();
    let pairs: std::collections::HashMap<_, _> = request.url().query_pairs().into_owned().collect();

    assert_eq!(pairs.len(), 5);
    assert_eq!(pairs["symbols"], "AAPL,MSFT");
    assert_eq!(pairs["status"], "all");
    assert_eq!(pairs["nested"], "false");
    assert_eq!(pairs["direction"], "asc");
    assert_eq!(pairs["limit"], "100");
}

#[test]
fn serialize_encodes_asset_query_when_filters_are_supplied() {
    let filters = ListAssetsRequest {
        asset_class: Some("us_equity".into()),
        attributes: Some("ipo,has_options".into()),
        ..Default::default()
    };
    let client = reqwest::blocking::Client::new();

    let request = client
        .get("http://localhost/v2/assets")
        .query(&filters)
        .build()
        .unwrap();
    let pairs: std::collections::HashMap<_, _> = request.url().query_pairs().into_owned().collect();

    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs["asset_class"], "us_equity");
    assert_eq!(pairs["attributes"], "ipo,has_options");
}

#[test]
fn serialize_omits_order_filters_when_request_is_default() {
    let request = ListOrdersRequest::default();

    let body = serde_json::to_value(request).unwrap();

    assert_eq!(body, json!({}));
}
