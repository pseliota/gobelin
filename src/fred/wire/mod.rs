mod observation;
mod series;

pub(crate) use observation::ObservationsPage;
pub(crate) use series::{SeriesList, SeriesPage};

use super::models::RealtimePeriod;
use crate::conversion::{self, ConversionError};

fn realtime_period(
    fields: [&'static str; 2],
    start: &str,
    end: &str,
) -> Result<RealtimePeriod, ConversionError> {
    Ok(RealtimePeriod {
        start: conversion::date(fields[0], start)?,
        end: conversion::date(fields[1], end)?,
    })
}

#[cfg(test)]
pub(crate) mod fixtures;
#[cfg(test)]
mod tests;
