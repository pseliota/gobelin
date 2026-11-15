use serde::Deserialize;

/// An open position from `/v2/positions`.
/// See <https://docs.alpaca.markets/reference/getallopenpositions>.
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    pub asset_id: String,
    pub symbol: String,
    pub exchange: String,
    pub asset_class: String,
    pub asset_marginable: Option<bool>,
    pub avg_entry_price: String,
    pub qty: String,
    pub side: String,
    pub cost_basis: String,
    // Alpaca's SDK permits missing valuation data, despite the stricter OpenAPI schema.
    pub market_value: Option<String>,
    pub unrealized_pl: Option<String>,
    pub unrealized_plpc: Option<String>,
    pub unrealized_intraday_pl: Option<String>,
    pub unrealized_intraday_plpc: Option<String>,
    pub current_price: Option<String>,
    pub lastday_price: Option<String>,
    pub change_today: Option<String>,
    pub qty_available: Option<String>,
    pub swap_rate: Option<String>,
    pub avg_entry_swap_rate: Option<String>,
    pub prev_swap_rate: Option<String>,
    pub usd: Option<UsdPositionValues>,
}

/// USD valuations supplied for local-currency trading accounts.
#[derive(Debug, Clone, Deserialize)]
pub struct UsdPositionValues {
    pub avg_entry_price: String,
    pub cost_basis: String,
    pub market_value: Option<String>,
    pub unrealized_pl: Option<String>,
    pub unrealized_plpc: Option<String>,
    pub unrealized_intraday_pl: Option<String>,
    pub unrealized_intraday_plpc: Option<String>,
    pub current_price: Option<String>,
    pub lastday_price: Option<String>,
    pub change_today: Option<String>,
}
