pub mod models;

use serde::de::DeserializeOwned;
use url::Url;

use self::models::{
    Account, Asset, CalendarDay, CalendarRequest, CancelOrderResult, Clock, CreateOrderRequest,
    ListAssetsRequest, ListOrdersRequest, Order, Position, ReplaceOrderRequest,
};
use super::credential_headers;
use crate::transport::{
    self, ClientError, HttpExecutor, HttpMethod, Request, Transport, TransportConfig, status,
};

const PAPER_URL: &str = "https://paper-api.alpaca.markets/v2/";

#[derive(Debug, Clone, Default)]
pub struct TradingClientConfig {
    pub transport: TransportConfig,
}

pub struct AlpacaTradingClient {
    transport: Transport,
}

impl AlpacaTradingClient {
    pub fn new(key: &str, secret: &str, config: TradingClientConfig) -> Result<Self, ClientError> {
        Self::with_executor(key, secret, config.transport.executor()?)
    }

    fn with_executor(
        key: &str,
        secret: &str,
        executor: Box<dyn HttpExecutor>,
    ) -> Result<Self, ClientError> {
        let base_url = Url::parse(PAPER_URL).expect("valid paper URL");
        let transport = Transport::new(base_url, credential_headers(key, secret)?, executor);
        Ok(Self { transport })
    }

    pub fn get_account(&self) -> Result<Account, ClientError> {
        self.json(self.request(HttpMethod::Get, &["account"])?)
    }

    pub fn list_assets(&self, filters: &ListAssetsRequest) -> Result<Vec<Asset>, ClientError> {
        self.json(self.request(HttpMethod::Get, &["assets"])?.query(filters)?)
    }

    pub fn get_asset(&self, symbol_or_id: &str) -> Result<Asset, ClientError> {
        self.json(self.request(HttpMethod::Get, &["assets", symbol_or_id])?)
    }

    pub fn list_positions(&self) -> Result<Vec<Position>, ClientError> {
        self.json(self.request(HttpMethod::Get, &["positions"])?)
    }

    pub fn get_position(&self, symbol_or_id: &str) -> Result<Position, ClientError> {
        self.json(self.request(HttpMethod::Get, &["positions", symbol_or_id])?)
    }

    pub fn list_orders(&self, filters: &ListOrdersRequest) -> Result<Vec<Order>, ClientError> {
        self.json(self.request(HttpMethod::Get, &["orders"])?.query(filters)?)
    }

    pub fn get_order(&self, order_id: &str, nested: bool) -> Result<Order, ClientError> {
        self.json(
            self.request(HttpMethod::Get, &["orders", order_id])?
                .query(&[("nested", nested)])?,
        )
    }

    pub fn get_order_by_client_id(&self, client_order_id: &str) -> Result<Order, ClientError> {
        self.json(
            self.request(HttpMethod::Get, &["orders:by_client_order_id"])?
                .query(&[("client_order_id", client_order_id)])?,
        )
    }

    pub fn create_order(&self, order: &CreateOrderRequest) -> Result<Order, ClientError> {
        self.json(self.request(HttpMethod::Post, &["orders"])?.json(order)?)
    }

    pub fn replace_order(
        &self,
        order_id: &str,
        order: &ReplaceOrderRequest,
    ) -> Result<Order, ClientError> {
        self.json(
            self.request(HttpMethod::Patch, &["orders", order_id])?
                .json(order)?,
        )
    }

    pub fn cancel_order(&self, order_id: &str) -> Result<(), ClientError> {
        self.transport.send(
            self.request(HttpMethod::Delete, &["orders", order_id])?,
            status::NO_CONTENT,
        )?;
        Ok(())
    }

    pub fn cancel_all_orders(&self) -> Result<Vec<CancelOrderResult>, ClientError> {
        let response = self.transport.send(
            self.request(HttpMethod::Delete, &["orders"])?,
            status::MULTI_STATUS,
        )?;
        transport::decode(&response)
    }

    pub fn get_clock(&self) -> Result<Clock, ClientError> {
        self.json(self.request(HttpMethod::Get, &["clock"])?)
    }

    pub fn get_calendar(&self, filters: &CalendarRequest) -> Result<Vec<CalendarDay>, ClientError> {
        self.json(
            self.request(HttpMethod::Get, &["calendar"])?
                .query(filters)?,
        )
    }

    fn request(&self, method: HttpMethod, segments: &[&str]) -> Result<Request, ClientError> {
        self.transport.request(method, segments)
    }

    fn json<T: DeserializeOwned>(&self, request: Request) -> Result<T, ClientError> {
        self.transport.json(request)
    }
}

#[cfg(test)]
mod tests;
