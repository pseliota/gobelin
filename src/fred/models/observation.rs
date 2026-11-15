use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Observation {
    pub realtime_start: String,
    pub realtime_end: String,
    pub date: String,
    pub value: String,
}

impl Observation {
    pub fn numeric_value(&self) -> Option<f64> {
        self.value
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ObservationsPage {
    pub realtime_start: String,
    pub realtime_end: String,
    pub observation_start: String,
    pub observation_end: String,
    pub units: String,
    pub output_type: u32,
    pub order_by: String,
    pub sort_order: String,
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
