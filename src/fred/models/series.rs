use std::fmt;

use super::{Frequency, RealtimePeriod, SearchOrderBy, SortOrder};
use crate::market::{Date, Timestamp};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeriesId(String);

impl SeriesId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SeriesId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for SeriesId {
    fn from(id: &str) -> Self {
        Self(id.to_owned())
    }
}

impl fmt::Display for SeriesId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SeasonalAdjustment {
    SeasonallyAdjusted,
    NotSeasonallyAdjusted,
    SeasonallyAdjustedAnnualRate,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    pub id: SeriesId,
    pub realtime: RealtimePeriod,
    pub title: String,
    pub observation_start: Date,
    pub observation_end: Date,
    pub frequency: Option<Frequency>,
    pub frequency_label: String,
    pub units: String,
    pub units_short: String,
    pub seasonal_adjustment: SeasonalAdjustment,
    pub last_updated: Timestamp,
    pub popularity: Option<u32>,
    pub group_popularity: Option<u32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesPage {
    pub realtime: RealtimePeriod,
    pub order_by: SearchOrderBy,
    pub sort_order: SortOrder,
    pub count: u32,
    pub offset: u32,
    pub limit: u32,
    pub series: Vec<Series>,
}

impl SeriesPage {
    pub fn next_offset(&self) -> Option<u32> {
        super::next_offset(self.offset, self.series.len(), self.count)
    }
}
