use crate::market::Date;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealtimePeriod {
    pub start: Date,
    pub end: Date,
}
