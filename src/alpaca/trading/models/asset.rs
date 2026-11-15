use serde::{Deserialize, Serialize};

/// Asset returned by `/v2/assets`.
/// See <https://docs.alpaca.markets/us/reference/get-v2-assets-1>.
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub id: String,
    #[serde(rename = "class")]
    pub asset_class: String,
    pub exchange: String,
    pub symbol: String,
    pub name: String,
    pub status: String,
    pub tradable: bool,
    pub marginable: bool,
    pub shortable: bool,
    pub fractionable: bool,
    pub easy_to_borrow: Option<bool>,
    pub borrow_status: Option<String>,
    pub cusip: Option<String>,
    pub attributes: Option<Vec<String>>,
    pub min_order_size: Option<String>,
    pub min_trade_increment: Option<String>,
    pub price_increment: Option<String>,
    pub margin_requirement_long: Option<String>,
    pub margin_requirement_short: Option<String>,
    // Deprecated numeric maintenance_margin_requirement is intentionally omitted.
}

/// Query parameters for `GET /v2/assets`. Lists use comma-separated strings.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListAssetsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<String>,
}
