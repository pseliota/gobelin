use serde::Deserialize;

/// Metadata of one economic data series.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Series {
    pub id: String,
    pub realtime_start: String,
    pub realtime_end: String,
    pub title: String,
    pub observation_start: String,
    pub observation_end: String,
    pub frequency: String,
    pub frequency_short: String,
    pub units: String,
    pub units_short: String,
    pub seasonal_adjustment: String,
    pub seasonal_adjustment_short: String,
    pub last_updated: String,
    pub popularity: Option<u32>,
    /// Sent only in search results.
    pub group_popularity: Option<u32>,
    pub notes: Option<String>,
}

/// One page of series search results.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SeriesPage {
    pub realtime_start: String,
    pub realtime_end: String,
    pub order_by: String,
    pub sort_order: String,
    pub count: u32,
    pub offset: u32,
    pub limit: u32,
    #[serde(rename = "seriess")]
    pub series: Vec<Series>,
}

impl SeriesPage {
    /// Returns the offset of the next page, or `None` when this page is empty or the last.
    pub fn next_offset(&self) -> Option<u32> {
        super::next_offset(self.offset, self.series.len(), self.count)
    }
}
