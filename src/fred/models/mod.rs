mod observation;
mod realtime;
mod request;
mod series;

pub use observation::{Observation, ObservationsPage};
pub use realtime::RealtimePeriod;
pub use request::{
    AggregationMethod, FilterVariable, Frequency, ObservationsRequest, SearchOrderBy,
    SearchRequest, SearchType, SortOrder, Units,
};
pub use series::{SeasonalAdjustment, Series, SeriesId, SeriesPage};

fn next_offset(offset: u32, items: usize, count: u32) -> Option<u32> {
    if items == 0 {
        return None;
    }
    let next = offset.checked_add(u32::try_from(items).ok()?)?;
    (next < count).then_some(next)
}

#[cfg(test)]
mod tests;
