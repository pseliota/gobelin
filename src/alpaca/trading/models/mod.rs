//! Wire models for Alpaca's core Trading API endpoints.
//!
//! Decimal strings, identifiers, timestamps, and response enum values retain their
//! wire representation. Unknown response fields are ignored. Request models omit
//! unset fields but do not validate numeric syntax or cross-field trading rules.

mod account;
mod asset;
mod cancellation;
mod market;
mod order;
mod position;

pub use account::Account;
pub use asset::{Asset, ListAssetsRequest};
pub use cancellation::CancelOrderResult;
pub use market::{CalendarDateType, CalendarDay, CalendarRequest, Clock};
pub use order::{
    AdvancedInstructions, CreateOrderRequest, ListOrdersRequest, Order, OrderClass,
    OrderLegRequest, OrderSide, OrderType, PositionIntent, QueryOrderStatus, ReplaceOrderRequest,
    SortDirection, StopLoss, TakeProfit, TimeInForce,
};
pub use position::{Position, UsdPositionValues};

#[cfg(test)]
mod tests;
