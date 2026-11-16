use serde::Deserialize;

use super::realtime_period;
use crate::conversion::{self, ConversionError};
use crate::fred::models::{self, SeasonalAdjustment};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Series {
    pub(crate) id: String,
    pub(crate) realtime_start: String,
    pub(crate) realtime_end: String,
    pub(crate) title: String,
    pub(crate) observation_start: String,
    pub(crate) observation_end: String,
    pub(crate) frequency: String,
    pub(crate) frequency_short: String,
    pub(crate) units: String,
    pub(crate) units_short: String,
    pub(crate) seasonal_adjustment_short: String,
    pub(crate) last_updated: String,
    pub(crate) popularity: Option<u32>,
    pub(crate) group_popularity: Option<u32>,
    pub(crate) notes: Option<String>,
}

impl TryFrom<Series> for models::Series {
    type Error = ConversionError;

    fn try_from(wire: Series) -> Result<Self, Self::Error> {
        Ok(Self {
            realtime: realtime_period(
                ["series.realtime_start", "series.realtime_end"],
                &wire.realtime_start,
                &wire.realtime_end,
            )?,
            observation_start: conversion::date(
                "series.observation_start",
                &wire.observation_start,
            )?,
            observation_end: conversion::date("series.observation_end", &wire.observation_end)?,
            frequency: conversion::known_code(&wire.frequency_short.to_ascii_lowercase()),
            seasonal_adjustment: seasonal_adjustment(wire.seasonal_adjustment_short),
            last_updated: conversion::timestamp("series.last_updated", &wire.last_updated)?,
            id: wire.id.into(),
            title: wire.title,
            frequency_label: wire.frequency,
            units: wire.units,
            units_short: wire.units_short,
            popularity: wire.popularity,
            group_popularity: wire.group_popularity,
            notes: wire.notes,
        })
    }
}

fn seasonal_adjustment(code: String) -> SeasonalAdjustment {
    match code.as_str() {
        "SA" => SeasonalAdjustment::SeasonallyAdjusted,
        "NSA" => SeasonalAdjustment::NotSeasonallyAdjusted,
        "SAAR" => SeasonalAdjustment::SeasonallyAdjustedAnnualRate,
        _ => SeasonalAdjustment::Unknown(code),
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct SeriesList {
    pub(crate) seriess: Vec<Series>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct SeriesPage {
    pub(crate) realtime_start: String,
    pub(crate) realtime_end: String,
    pub(crate) order_by: String,
    pub(crate) sort_order: String,
    pub(crate) count: u32,
    pub(crate) offset: u32,
    pub(crate) limit: u32,
    pub(crate) seriess: Vec<Series>,
}

impl TryFrom<SeriesPage> for models::SeriesPage {
    type Error = ConversionError;

    fn try_from(wire: SeriesPage) -> Result<Self, Self::Error> {
        Ok(Self {
            realtime: realtime_period(
                ["search.realtime_start", "search.realtime_end"],
                &wire.realtime_start,
                &wire.realtime_end,
            )?,
            order_by: conversion::code("search.order_by", &wire.order_by)?,
            sort_order: conversion::code("search.sort_order", &wire.sort_order)?,
            count: wire.count,
            offset: wire.offset,
            limit: wire.limit,
            series: wire
                .seriess
                .into_iter()
                .map(models::Series::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
