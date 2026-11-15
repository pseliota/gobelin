use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Trade {
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "x")]
    pub exchange: String,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: u32,
    #[serde(rename = "c", default)]
    pub conditions: Vec<String>,
    #[serde(rename = "i")]
    pub id: u64,
    #[serde(rename = "z")]
    pub tape: String,
    #[serde(rename = "u")]
    pub update: Option<String>,
}
