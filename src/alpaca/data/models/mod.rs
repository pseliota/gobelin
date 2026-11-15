//! Wire models for Alpaca's Market Data API.
//!
//! Prices are JSON numbers and are held as `f64`. Timestamps stay RFC 3339 strings.
//! Unknown response fields are ignored.

mod bar;
mod quote;
mod request;
mod snapshot;
mod trade;

pub use bar::{Bar, BarsPage};
pub use quote::Quote;
pub use request::{Adjustment, BarsRequest, Feed, SortDirection};
pub use snapshot::Snapshot;
pub use trade::Trade;

#[cfg(test)]
mod tests;
