use serde::{Deserialize, Serialize};

/// Order response, including nested bracket or multi-leg orders.
/// Empty strings in multi-leg responses are preserved rather than normalized.
/// See <https://docs.alpaca.markets/reference/postorder>.
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: String,
    pub client_order_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub submitted_at: String,
    pub filled_at: Option<String>,
    pub expired_at: Option<String>,
    pub expires_at: Option<String>,
    pub canceled_at: Option<String>,
    pub failed_at: Option<String>,
    pub replaced_at: Option<String>,
    pub replaced_by: Option<String>,
    pub replaces: Option<String>,
    pub asset_id: Option<String>,
    pub symbol: Option<String>,
    pub asset_class: Option<String>,
    pub notional: Option<String>,
    pub qty: Option<String>,
    pub filled_qty: Option<String>,
    pub filled_avg_price: Option<String>,
    pub order_class: Option<String>,
    #[serde(rename = "type")]
    pub order_type: Option<String>,
    pub side: Option<String>,
    pub time_in_force: String,
    pub limit_price: Option<String>,
    pub stop_price: Option<String>,
    pub status: String,
    pub extended_hours: bool,
    pub legs: Option<Vec<Order>>,
    pub trail_percent: Option<String>,
    pub trail_price: Option<String>,
    pub hwm: Option<String>,
    pub position_intent: Option<String>,
    pub ratio_qty: Option<String>,
    pub group_id: Option<String>,
    pub group_role: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    TrailingStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    Day,
    Gtc,
    Opg,
    Cls,
    Ioc,
    Fok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderClass {
    Simple,
    Bracket,
    Oco,
    Oto,
    Mleg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionIntent {
    BuyToOpen,
    BuyToClose,
    SellToOpen,
    SellToClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryOrderStatus {
    Open,
    Closed,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// JSON body for `POST /v2/orders`, not a validated trading instruction.
/// Exactly one of qty/notional is needed; other requirements depend on order class/type.
#[derive(Debug, Clone, Serialize)]
pub struct CreateOrderRequest {
    #[serde(rename = "type")]
    pub order_type: OrderType,
    pub time_in_force: TimeInForce,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<OrderSide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail_percent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_hours: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_class: Option<OrderClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub take_profit: Option<TakeProfit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_loss: Option<StopLoss>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_intent: Option<PositionIntent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legs: Option<Vec<OrderLegRequest>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_instructions: Option<AdvancedInstructions>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TakeProfit {
    pub limit_price: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopLoss {
    pub stop_price: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderLegRequest {
    pub symbol: String,
    pub ratio_qty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<OrderSide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_intent: Option<PositionIntent>,
}

/// Elite Smart Router parameters; eligibility and algorithm rules are not validated here.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AdvancedInstructions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_qty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_percentage: Option<String>,
}

/// JSON body for `PATCH /v2/orders/{order_id}`. Unset fields remain unchanged.
/// Notional is IPO-only and mutually exclusive with qty.
/// See <https://docs.alpaca.markets/us/reference/patchorderbyorderid-1>.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ReplaceOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advanced_instructions: Option<AdvancedInstructions>,
}

/// Query parameters for `GET /v2/orders`. Symbols and asset classes are comma-separated.
/// Timestamp filters and order-ID filters must not be combined.
/// See <https://docs.alpaca.markets/us/reference/getallorders-1>.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<QueryOrderStatus>,
    /// Maximum 500; Alpaca defaults to 50 when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SortDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nested: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side: Option<OrderSide>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbols: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_order_id: Option<String>,
}
