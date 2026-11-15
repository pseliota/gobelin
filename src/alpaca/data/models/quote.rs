use serde::Deserialize;

/// National best bid and offer.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Quote {
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "ax")]
    pub ask_exchange: String,
    #[serde(rename = "ap")]
    pub ask_price: f64,
    #[serde(rename = "as")]
    pub ask_size: u32,
    #[serde(rename = "bx")]
    pub bid_exchange: String,
    #[serde(rename = "bp")]
    pub bid_price: f64,
    #[serde(rename = "bs")]
    pub bid_size: u32,
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    #[serde(rename = "z")]
    pub tape: String,
}
