use serde::{Serialize, Serializer};

use crate::alpaca::models::SortDirection;

/// Source of stock market data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Feed {
    Iex,
    Sip,
    DelayedSip,
    Otc,
    Boats,
    Overnight,
}

/// Corporate action adjustment applied to bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Adjustment {
    Raw,
    Split,
    Dividend,
    SpinOff,
    All,
}

impl Adjustment {
    fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Split => "split",
            Self::Dividend => "dividend",
            Self::SpinOff => "spin-off",
            Self::All => "all",
        }
    }
}

impl Serialize for Adjustment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// Query parameters for historical bars. Values are sent as supplied.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BarsRequest {
    #[serde(serialize_with = "comma_joined_symbols")]
    pub symbols: Vec<String>,
    pub timeframe: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "comma_joined_adjustments"
    )]
    pub adjustment: Vec<Adjustment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asof: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<SortDirection>,
}

fn comma_joined_symbols<S: Serializer>(
    symbols: &[String],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&symbols.join(","))
}

fn comma_joined_adjustments<S: Serializer>(
    adjustments: &[Adjustment],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let names: Vec<&str> = adjustments.iter().map(|a| a.as_str()).collect();
    serializer.serialize_str(&names.join(","))
}
