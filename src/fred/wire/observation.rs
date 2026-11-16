use serde::Deserialize;

use super::realtime_period;
use crate::conversion::{self, ConversionError};
use crate::fred::models;

const MISSING_VALUE: &str = ".";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Observation {
    pub(crate) realtime_start: String,
    pub(crate) realtime_end: String,
    pub(crate) date: String,
    pub(crate) value: String,
}

impl TryFrom<Observation> for models::Observation {
    type Error = ConversionError;

    fn try_from(wire: Observation) -> Result<Self, Self::Error> {
        let value = match wire.value.as_str() {
            MISSING_VALUE => None,
            value => Some(conversion::number("observation.value", value)?),
        };
        Ok(Self {
            date: conversion::date("observation.date", &wire.date)?,
            value,
            realtime: realtime_period(
                ["observation.realtime_start", "observation.realtime_end"],
                &wire.realtime_start,
                &wire.realtime_end,
            )?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct ObservationsPage {
    pub(crate) realtime_start: String,
    pub(crate) realtime_end: String,
    pub(crate) observation_start: String,
    pub(crate) observation_end: String,
    pub(crate) units: String,
    pub(crate) output_type: u32,
    pub(crate) sort_order: String,
    pub(crate) count: u32,
    pub(crate) offset: u32,
    pub(crate) limit: u32,
    pub(crate) observations: Vec<Observation>,
}

impl TryFrom<ObservationsPage> for models::ObservationsPage {
    type Error = ConversionError;

    fn try_from(wire: ObservationsPage) -> Result<Self, Self::Error> {
        Ok(Self {
            realtime: realtime_period(
                ["observations.realtime_start", "observations.realtime_end"],
                &wire.realtime_start,
                &wire.realtime_end,
            )?,
            observation_start: conversion::date(
                "observations.observation_start",
                &wire.observation_start,
            )?,
            observation_end: conversion::date(
                "observations.observation_end",
                &wire.observation_end,
            )?,
            units: conversion::code("observations.units", &wire.units)?,
            output_type: wire.output_type,
            sort_order: conversion::code("observations.sort_order", &wire.sort_order)?,
            count: wire.count,
            offset: wire.offset,
            limit: wire.limit,
            observations: wire
                .observations
                .into_iter()
                .map(models::Observation::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}
