//! Wire models for FRED API version 1.
//!
//! Dates stay `YYYY-MM-DD` strings. Observation values stay strings as sent.
//! Unknown response fields are ignored.

mod observation;
mod request;
mod series;

pub use observation::{Observation, ObservationsPage};
pub use request::{
    AggregationMethod, FilterVariable, Frequency, ObservationsRequest, SearchOrderBy,
    SearchRequest, SearchType, SortOrder, Units,
};
pub use series::{Series, SeriesPage};

/// Returns the offset after a page of `items`, or `None` when nothing remains.
fn next_offset(offset: u32, items: usize, count: u32) -> Option<u32> {
    if items == 0 {
        return None;
    }
    let next = offset.checked_add(u32::try_from(items).ok()?)?;
    (next < count).then_some(next)
}

#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
mod tests;
