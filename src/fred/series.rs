use serde::Deserialize;

use super::FredClient;
use super::models::{ObservationsPage, ObservationsRequest, SearchRequest, Series, SeriesPage};
use crate::transport::ClientError;

#[derive(Deserialize)]
struct SeriesList {
    seriess: Vec<Series>,
}

impl FredClient {
    pub fn get_series(&self, series_id: &str) -> Result<Series, ClientError> {
        require(series_id)?;
        let list: SeriesList = self.fetch(&["fred", "series"], &[("series_id", series_id)])?;
        list.seriess
            .into_iter()
            .next()
            .ok_or(ClientError::InvalidResponse)
    }

    /// Returns one page of observations; the server's default `limit` is 100000.
    /// Pass the page's [`ObservationsPage::next_offset`] as `offset` to continue.
    ///
    /// Without `realtime_start` and `realtime_end` the latest revision is returned,
    /// which can differ from the values known at an earlier date.
    pub fn get_observations(
        &self,
        req: &ObservationsRequest,
    ) -> Result<ObservationsPage, ClientError> {
        require(&req.series_id)?;
        self.fetch(&["fred", "series", "observations"], req)
    }

    /// Returns one page of series matching the search.
    /// Pass the page's [`SeriesPage::next_offset`] as `offset` to continue.
    pub fn search_series(&self, req: &SearchRequest) -> Result<SeriesPage, ClientError> {
        require(&req.search_text)?;
        self.fetch(&["fred", "series", "search"], req)
    }
}

fn require(value: &str) -> Result<(), ClientError> {
    if value.trim().is_empty() {
        return Err(ClientError::InvalidRequest);
    }
    Ok(())
}
