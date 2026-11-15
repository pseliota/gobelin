use std::collections::HashMap;

use serde::Deserialize;

use super::AlpacaDataClient;
use super::models::{Bar, BarsPage, BarsRequest, Quote, Snapshot, Trade};
use super::pages::{Page, Pages};
use crate::transport::{ClientError, GetRequest};

#[derive(Deserialize)]
struct LatestBars {
    bars: HashMap<String, Bar>,
}

#[derive(Deserialize)]
struct LatestQuotes {
    quotes: HashMap<String, Quote>,
}

#[derive(Deserialize)]
struct LatestTrades {
    trades: HashMap<String, Trade>,
}

impl Page for BarsPage {
    fn next_page_token(&self) -> Option<&str> {
        self.next_page_token.as_deref()
    }
}

impl AlpacaDataClient {
    /// Returns one page of historical bars; pass the previous page's token to continue.
    ///
    /// The request's `limit` counts bars across all symbols, so a page of a
    /// multi-symbol request can omit symbols that appear on later pages.
    pub fn get_stock_bars(
        &self,
        req: &BarsRequest,
        page_token: Option<&str>,
    ) -> Result<BarsPage, ClientError> {
        validate_symbols(&req.symbols)?;
        let mut request = self.stock_request(&["bars"])?.query(req);
        if let Some(token) = page_token {
            request = request.query(&[("page_token", token)]);
        }
        self.transport.fetch(request)
    }

    /// Returns every page of historical bars, fetching each when the iterator is advanced.
    /// The iterator ends after the last page or after yielding the first error.
    pub fn stock_bars_pages<'a>(
        &'a self,
        req: &'a BarsRequest,
    ) -> impl Iterator<Item = Result<BarsPage, ClientError>> + 'a {
        Pages::new(move |token| self.get_stock_bars(req, token))
    }

    pub fn get_latest_stock_bars(
        &self,
        symbols: &[&str],
    ) -> Result<HashMap<String, Bar>, ClientError> {
        let response: LatestBars = self
            .transport
            .fetch(self.symbols_request(&["bars", "latest"], symbols)?)?;
        Ok(response.bars)
    }

    pub fn get_latest_stock_quotes(
        &self,
        symbols: &[&str],
    ) -> Result<HashMap<String, Quote>, ClientError> {
        let response: LatestQuotes = self
            .transport
            .fetch(self.symbols_request(&["quotes", "latest"], symbols)?)?;
        Ok(response.quotes)
    }

    pub fn get_latest_stock_trades(
        &self,
        symbols: &[&str],
    ) -> Result<HashMap<String, Trade>, ClientError> {
        let response: LatestTrades = self
            .transport
            .fetch(self.symbols_request(&["trades", "latest"], symbols)?)?;
        Ok(response.trades)
    }

    pub fn get_stock_snapshots(
        &self,
        symbols: &[&str],
    ) -> Result<HashMap<String, Snapshot>, ClientError> {
        self.transport
            .fetch(self.symbols_request(&["snapshots"], symbols)?)
    }

    fn symbols_request(
        &self,
        segments: &[&str],
        symbols: &[&str],
    ) -> Result<GetRequest, ClientError> {
        validate_symbols(symbols)?;
        let request = self.stock_request(segments)?;
        Ok(request.query(&[("symbols", symbols.join(","))]))
    }

    /// Starts a GET under `v2/stocks` carrying the configured feed.
    fn stock_request(&self, segments: &[&str]) -> Result<GetRequest, ClientError> {
        let path = [&["v2", "stocks"], segments].concat();
        let mut request = self.transport.get(&path)?;
        if let Some(feed) = self.feed {
            request = request.query(&[("feed", feed)]);
        }
        Ok(request)
    }
}

fn validate_symbols<S: AsRef<str>>(symbols: &[S]) -> Result<(), ClientError> {
    if symbols.is_empty() || symbols.iter().any(|s| s.as_ref().trim().is_empty()) {
        return Err(ClientError::InvalidRequest);
    }
    Ok(())
}
