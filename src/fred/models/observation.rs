use super::{RealtimePeriod, SortOrder, Units};
use crate::market::Date;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    pub date: Date,
    pub value: Option<f64>,
    pub realtime: RealtimePeriod,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObservationsPage {
    pub realtime: RealtimePeriod,
    pub observation_start: Date,
    pub observation_end: Date,
    pub units: Units,
    pub output_type: u32,
    pub sort_order: SortOrder,
    pub count: u32,
    pub offset: u32,
    pub limit: u32,
    pub observations: Vec<Observation>,
}

impl ObservationsPage {
    pub fn next_offset(&self) -> Option<u32> {
        super::next_offset(self.offset, self.observations.len(), self.count)
    }
}
