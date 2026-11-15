//! Blocking HTTP access to Alpaca's paper Trading API.

pub mod models;

use reqwest::{Method, StatusCode, Url, blocking::RequestBuilder};
use serde::de::DeserializeOwned;

use self::models::{
    Account, Asset, CalendarDay, CalendarRequest, CancelOrderResult, Clock, CreateOrderRequest,
    ListAssetsRequest, ListOrdersRequest, Order, Position, ReplaceOrderRequest,
};
use super::ClientError;
use super::transport::{self, Transport};

const PAPER_URL: &str = "https://paper-api.alpaca.markets/v2/";

/// A reusable paper-only client with a 20-second request timeout.
/// Redirects and retries are disabled. Credentials and response bodies are not logged.
/// Request models are serialized as supplied; Alpaca validates trading constraints.
pub struct AlpacaTradingClient {
    transport: Transport,
}

impl AlpacaTradingClient {
    pub fn new(key: &str, secret: &str) -> Result<Self, ClientError> {
        Self::build(key, secret, Url::parse(PAPER_URL).expect("valid paper URL"))
    }

    fn build(key: &str, secret: &str, base_url: Url) -> Result<Self, ClientError> {
        let transport = Transport::new(key, secret, base_url)?;
        Ok(Self { transport })
    }

    pub fn get_account(&self) -> Result<Account, ClientError> {
        self.json(self.request(Method::GET, &["account"])?)
    }

    pub fn list_assets(&self, filters: &ListAssetsRequest) -> Result<Vec<Asset>, ClientError> {
        self.json(self.request(Method::GET, &["assets"])?.query(filters))
    }

    pub fn get_asset(&self, symbol_or_id: &str) -> Result<Asset, ClientError> {
        self.json(self.request(Method::GET, &["assets", symbol_or_id])?)
    }

    pub fn list_positions(&self) -> Result<Vec<Position>, ClientError> {
        self.json(self.request(Method::GET, &["positions"])?)
    }

    pub fn get_position(&self, symbol_or_id: &str) -> Result<Position, ClientError> {
        self.json(self.request(Method::GET, &["positions", symbol_or_id])?)
    }

    /// Returns one page. Pagination remains explicit through the supplied filters.
    pub fn list_orders(&self, filters: &ListOrdersRequest) -> Result<Vec<Order>, ClientError> {
        self.json(self.request(Method::GET, &["orders"])?.query(filters))
    }

    pub fn get_order(&self, order_id: &str, nested: bool) -> Result<Order, ClientError> {
        self.json(
            self.request(Method::GET, &["orders", order_id])?
                .query(&[("nested", nested)]),
        )
    }

    pub fn get_order_by_client_id(&self, client_order_id: &str) -> Result<Order, ClientError> {
        self.json(
            self.request(Method::GET, &["orders:by_client_order_id"])?
                .query(&[("client_order_id", client_order_id)]),
        )
    }

    /// Submits a paper order. A transport failure does not prove submission failed;
    /// reconcile using a caller-supplied client_order_id before retrying.
    pub fn create_order(&self, order: &CreateOrderRequest) -> Result<Order, ClientError> {
        self.json(self.request(Method::POST, &["orders"])?.json(order))
    }

    /// Requests replacement; acceptance does not guarantee the old order was replaced.
    pub fn replace_order(
        &self,
        order_id: &str,
        order: &ReplaceOrderRequest,
    ) -> Result<Order, ClientError> {
        self.json(
            self.request(Method::PATCH, &["orders", order_id])?
                .json(order),
        )
    }

    /// Requests cancellation. HTTP 204 acknowledges the request, not final order state.
    pub fn cancel_order(&self, order_id: &str) -> Result<(), ClientError> {
        self.transport.send(
            self.request(Method::DELETE, &["orders", order_id])?,
            StatusCode::NO_CONTENT,
        )?;
        Ok(())
    }

    /// Each result must be checked individually; HTTP 207 is not universal success.
    pub fn cancel_all_orders(&self) -> Result<Vec<CancelOrderResult>, ClientError> {
        let response = self.transport.send(
            self.request(Method::DELETE, &["orders"])?,
            StatusCode::MULTI_STATUS,
        )?;
        transport::decode(response)
    }

    pub fn get_clock(&self) -> Result<Clock, ClientError> {
        self.json(self.request(Method::GET, &["clock"])?)
    }

    pub fn get_calendar(&self, filters: &CalendarRequest) -> Result<Vec<CalendarDay>, ClientError> {
        self.json(self.request(Method::GET, &["calendar"])?.query(filters))
    }

    fn request(&self, method: Method, segments: &[&str]) -> Result<RequestBuilder, ClientError> {
        self.transport.request(method, segments)
    }

    fn json<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, ClientError> {
        self.transport.json(request)
    }
}

#[cfg(test)]
mod tests;
