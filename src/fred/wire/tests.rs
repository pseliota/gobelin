use serde_json::{Value, json};

use super::fixtures::{OBSERVATIONS, SEARCH, SERIES};
use super::observation::Observation;
use super::series::Series;
use super::*;
use crate::conversion::ValueKind;
use crate::fred::models::{self, Frequency, SearchOrderBy, SeasonalAdjustment, SortOrder, Units};
use crate::market::Date;

fn date(year: i16, month: i8, day: i8) -> Date {
    Date::new(year, month, day).unwrap()
}

fn realtime(year: i16, month: i8, day: i8) -> RealtimePeriod {
    RealtimePeriod {
        start: date(year, month, day),
        end: date(year, month, day),
    }
}

fn first_series(document: &str) -> Value {
    let mut value: Value = serde_json::from_str(document).unwrap();
    value["seriess"][0].take()
}

fn series() -> Series {
    serde_json::from_value(first_series(SERIES)).unwrap()
}

fn observation(value: &str) -> Observation {
    Observation {
        realtime_start: "2013-08-14".into(),
        realtime_end: "2013-08-14".into(),
        date: "1929-01-01".into(),
        value: value.into(),
    }
}

fn observations_page() -> ObservationsPage {
    serde_json::from_str(OBSERVATIONS).unwrap()
}

fn series_page() -> SeriesPage {
    serde_json::from_str(SEARCH).unwrap()
}

fn failure(field: &'static str, expected: ValueKind) -> ConversionError {
    ConversionError::new(field, expected)
}

#[test]
fn deserialize_maps_series_fields_when_response_is_series_fixture() {
    let response = first_series(SERIES);

    let series: Series = serde_json::from_value(response).unwrap();

    assert_eq!(
        series,
        Series {
            id: "GNPCA".into(),
            realtime_start: "2013-08-14".into(),
            realtime_end: "2013-08-14".into(),
            title: "Real Gross National Product".into(),
            observation_start: "1929-01-01".into(),
            observation_end: "2012-01-01".into(),
            frequency: "Annual".into(),
            frequency_short: "A".into(),
            units: "Billions of Chained 2009 Dollars".into(),
            units_short: "Bil. of Chn. 2009 $".into(),
            seasonal_adjustment_short: "NSA".into(),
            last_updated: "2013-07-31 09:26:16-05".into(),
            popularity: Some(39),
            group_popularity: None,
            notes: Some("BEA Account Code: A001RX1".into()),
        }
    );
}

#[test]
fn deserialize_leaves_optional_series_fields_unknown_when_they_are_absent() {
    let mut response = first_series(SERIES);
    let fields = response.as_object_mut().unwrap();
    fields.remove("notes");
    fields.remove("popularity");

    let series: Series = serde_json::from_value(response).unwrap();

    assert_eq!(series.notes, None);
    assert_eq!(series.popularity, None);
}

#[test]
fn deserialize_leaves_optional_series_fields_unknown_when_they_are_null() {
    let mut response = first_series(SERIES);
    response["notes"] = json!(null);
    response["popularity"] = json!(null);
    response["group_popularity"] = json!(null);

    let series: Series = serde_json::from_value(response).unwrap();

    assert_eq!(series.notes, None);
    assert_eq!(series.popularity, None);
    assert_eq!(series.group_popularity, None);
}

#[test]
fn deserialize_parses_observations_page_when_response_is_observations_fixture() {
    let page: ObservationsPage = serde_json::from_str(OBSERVATIONS).unwrap();

    assert_eq!(page.realtime_start, "2013-08-14");
    assert_eq!(page.realtime_end, "2013-08-14");
    assert_eq!(page.observation_start, "1776-07-04");
    assert_eq!(page.observation_end, "9999-12-31");
    assert_eq!(page.units, "lin");
    assert_eq!(page.output_type, 1);
    assert_eq!(page.sort_order, "asc");
    assert_eq!(page.count, 84);
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, 100000);
    assert_eq!(
        page.observations,
        vec![
            Observation {
                realtime_start: "2013-08-14".into(),
                realtime_end: "2013-08-14".into(),
                date: "1929-01-01".into(),
                value: "1065.9".into(),
            },
            Observation {
                realtime_start: "2013-08-14".into(),
                realtime_end: "2013-08-14".into(),
                date: "1930-01-01".into(),
                value: "975.5".into(),
            },
        ]
    );
}

#[test]
fn deserialize_parses_series_page_when_response_is_search_fixture() {
    let page: SeriesPage = serde_json::from_str(SEARCH).unwrap();

    assert_eq!(page.realtime_start, "2017-08-01");
    assert_eq!(page.realtime_end, "2017-08-01");
    assert_eq!(page.order_by, "search_rank");
    assert_eq!(page.sort_order, "desc");
    assert_eq!(page.count, 32);
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, 1000);
    assert_eq!(page.seriess.len(), 1);
    assert_eq!(page.seriess[0].id, "MSIM2");
    assert_eq!(page.seriess[0].popularity, Some(34));
    assert_eq!(page.seriess[0].group_popularity, Some(33));
}

#[test]
fn try_from_types_observation_when_value_is_numeric() {
    let wire = observation("1065.9");

    let converted = models::Observation::try_from(wire);

    assert_eq!(
        converted,
        Ok(models::Observation {
            date: date(1929, 1, 1),
            value: Some(1065.9),
            realtime: realtime(2013, 8, 14),
        })
    );
}

#[test]
fn try_from_leaves_observation_value_empty_when_value_is_missing_marker() {
    let wire = observation(".");

    let converted = models::Observation::try_from(wire).unwrap();

    assert_eq!(converted.value, None);
}

#[test]
fn try_from_rejects_observation_when_value_is_malformed() {
    let cases = ["", "n/a", "NaN", "inf", "1,000", ".."];

    for text in cases {
        let converted = models::Observation::try_from(observation(text));

        assert_eq!(
            converted,
            Err(failure("observation.value", ValueKind::Number)),
            "{text:?}"
        );
    }
}

#[test]
fn try_from_rejects_observation_when_date_is_malformed() {
    let mut wire = observation("1.0");
    wire.date = "1929-02-30".into();

    let converted = models::Observation::try_from(wire);

    assert_eq!(converted, Err(failure("observation.date", ValueKind::Date)));
}

#[test]
fn try_from_rejects_observation_when_realtime_end_is_malformed() {
    let mut wire = observation("1.0");
    wire.realtime_end = "today".into();

    let converted = models::Observation::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("observation.realtime_end", ValueKind::Date))
    );
}

#[test]
fn try_from_types_observations_page_when_response_is_observations_fixture() {
    let wire = observations_page();

    let page = models::ObservationsPage::try_from(wire).unwrap();

    assert_eq!(
        page,
        models::ObservationsPage {
            realtime: realtime(2013, 8, 14),
            observation_start: date(1776, 7, 4),
            observation_end: date(9999, 12, 31),
            units: Units::Levels,
            output_type: 1,
            sort_order: SortOrder::Asc,
            count: 84,
            offset: 0,
            limit: 100_000,
            observations: vec![
                models::Observation {
                    date: date(1929, 1, 1),
                    value: Some(1065.9),
                    realtime: realtime(2013, 8, 14),
                },
                models::Observation {
                    date: date(1930, 1, 1),
                    value: Some(975.5),
                    realtime: realtime(2013, 8, 14),
                },
            ],
        }
    );
}

#[test]
fn try_from_rejects_observations_page_when_units_are_unknown() {
    let mut wire = observations_page();
    wire.units = "furlongs".into();

    let converted = models::ObservationsPage::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("observations.units", ValueKind::Code))
    );
}

#[test]
fn try_from_rejects_observations_page_when_sort_order_is_unknown() {
    let mut wire = observations_page();
    wire.sort_order = "sideways".into();

    let converted = models::ObservationsPage::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("observations.sort_order", ValueKind::Code))
    );
}

#[test]
fn try_from_rejects_observations_page_when_an_observation_is_malformed() {
    let mut wire = observations_page();
    wire.observations[1].value = "n/a".into();

    let converted = models::ObservationsPage::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("observation.value", ValueKind::Number))
    );
}

#[test]
fn try_from_types_series_when_response_is_series_fixture() {
    let wire = series();

    let converted = models::Series::try_from(wire);

    assert_eq!(
        converted,
        Ok(models::Series {
            id: "GNPCA".into(),
            realtime: realtime(2013, 8, 14),
            title: "Real Gross National Product".into(),
            observation_start: date(1929, 1, 1),
            observation_end: date(2012, 1, 1),
            frequency: Some(Frequency::Annual),
            frequency_label: "Annual".into(),
            units: "Billions of Chained 2009 Dollars".into(),
            units_short: "Bil. of Chn. 2009 $".into(),
            seasonal_adjustment: SeasonalAdjustment::NotSeasonallyAdjusted,
            last_updated: "2013-07-31T14:26:16Z".parse().unwrap(),
            popularity: Some(39),
            group_popularity: None,
            notes: Some("BEA Account Code: A001RX1".into()),
        })
    );
}

#[test]
fn try_from_maps_series_frequency_when_short_code_is_known() {
    let cases = [
        ("D", Frequency::Daily),
        ("W", Frequency::Weekly),
        ("BW", Frequency::Biweekly),
        ("M", Frequency::Monthly),
        ("Q", Frequency::Quarterly),
        ("SA", Frequency::Semiannual),
        ("A", Frequency::Annual),
    ];

    for (code, expected) in cases {
        let mut wire = series();
        wire.frequency_short = code.into();

        let converted = models::Series::try_from(wire).unwrap();

        assert_eq!(converted.frequency, Some(expected), "{code}");
    }
}

#[test]
fn try_from_keeps_series_frequency_label_when_short_code_is_unknown() {
    let mut wire = series();
    wire.frequency = "Not Applicable".into();
    wire.frequency_short = "NA".into();

    let converted = models::Series::try_from(wire).unwrap();

    assert_eq!(converted.frequency, None);
    assert_eq!(converted.frequency_label, "Not Applicable");
}

#[test]
fn try_from_maps_series_seasonal_adjustment_when_short_code_is_known() {
    let cases = [
        ("SA", SeasonalAdjustment::SeasonallyAdjusted),
        ("NSA", SeasonalAdjustment::NotSeasonallyAdjusted),
        ("SAAR", SeasonalAdjustment::SeasonallyAdjustedAnnualRate),
    ];

    for (code, expected) in cases {
        let mut wire = series();
        wire.seasonal_adjustment_short = code.into();

        let converted = models::Series::try_from(wire).unwrap();

        assert_eq!(converted.seasonal_adjustment, expected, "{code}");
    }
}

#[test]
fn try_from_keeps_series_seasonal_adjustment_code_when_code_is_unknown() {
    let mut wire = series();
    wire.seasonal_adjustment_short = "SSA".into();

    let converted = models::Series::try_from(wire).unwrap();

    assert_eq!(
        converted.seasonal_adjustment,
        SeasonalAdjustment::Unknown("SSA".into())
    );
}

#[test]
fn try_from_rejects_series_when_last_updated_is_malformed() {
    let mut wire = series();
    wire.last_updated = "2013-07-31 09:26:16".into();

    let converted = models::Series::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("series.last_updated", ValueKind::Timestamp))
    );
}

#[test]
fn try_from_rejects_series_when_observation_start_is_malformed() {
    let mut wire = series();
    wire.observation_start = "long ago".into();

    let converted = models::Series::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("series.observation_start", ValueKind::Date))
    );
}

#[test]
fn try_from_types_series_page_when_response_is_search_fixture() {
    let wire = series_page();

    let page = models::SeriesPage::try_from(wire).unwrap();

    assert_eq!(page.realtime, realtime(2017, 8, 1));
    assert_eq!(page.order_by, SearchOrderBy::SearchRank);
    assert_eq!(page.sort_order, SortOrder::Desc);
    assert_eq!(page.count, 32);
    assert_eq!(page.offset, 0);
    assert_eq!(page.limit, 1000);
    assert_eq!(page.series.len(), 1);
    assert_eq!(page.series[0].id.as_str(), "MSIM2");
    assert_eq!(page.series[0].frequency, Some(Frequency::Monthly));
    assert_eq!(
        page.series[0].seasonal_adjustment,
        SeasonalAdjustment::SeasonallyAdjusted
    );
    assert_eq!(
        page.series[0].last_updated,
        "2014-01-17T13:16:44Z".parse().unwrap()
    );
}

#[test]
fn try_from_rejects_series_page_when_ordering_is_unknown() {
    let mut wire = series_page();
    wire.order_by = "relevance".into();

    let converted = models::SeriesPage::try_from(wire);

    assert_eq!(converted, Err(failure("search.order_by", ValueKind::Code)));
}

#[test]
fn try_from_rejects_series_page_when_a_series_is_malformed() {
    let mut wire = series_page();
    wire.seriess[0].realtime_start = "now".into();

    let converted = models::SeriesPage::try_from(wire);

    assert_eq!(
        converted,
        Err(failure("series.realtime_start", ValueKind::Date))
    );
}
