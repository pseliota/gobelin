use super::FredClient;
use super::models::{ObservationsPage, ObservationsRequest, SearchRequest, Series, SeriesPage};
use super::wire;
use crate::transport::ClientError;

impl FredClient {
    pub fn get_series(&self, series_id: &str) -> Result<Series, ClientError> {
        require(series_id)?;
        let list: wire::SeriesList =
            self.fetch(&["fred", "series"], &[("series_id", series_id)])?;
        let series = list
            .seriess
            .into_iter()
            .next()
            .ok_or(ClientError::InvalidResponse)?;
        Ok(series.try_into()?)
    }

    pub fn get_observations(
        &self,
        req: &ObservationsRequest,
    ) -> Result<ObservationsPage, ClientError> {
        require(&req.series_id)?;
        let page: wire::ObservationsPage = self.fetch(&["fred", "series", "observations"], req)?;
        Ok(page.try_into()?)
    }

    pub fn search_series(&self, req: &SearchRequest) -> Result<SeriesPage, ClientError> {
        require(&req.search_text)?;
        let page: wire::SeriesPage = self.fetch(&["fred", "series", "search"], req)?;
        Ok(page.try_into()?)
    }
}

fn require(value: &str) -> Result<(), ClientError> {
    if value.trim().is_empty() {
        return Err(ClientError::InvalidRequest);
    }
    Ok(())
}
