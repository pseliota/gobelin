use super::fixtures::{OBSERVATIONS, SEARCH, SERIES};
use super::*;
use serde::Serialize;
use serde_json::{Value, json};

fn query_pairs<Q: Serialize>(request: &Q) -> Vec<(String, String)> {
    reqwest::blocking::Client::new()
        .get("http://localhost/")
        .query(request)
        .build()
        .unwrap()
        .url()
        .query_pairs()
        .into_owned()
        .collect()
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn wire_name<T: Serialize>(value: T) -> Value {
    serde_json::to_value(value).unwrap()
}

fn first_series(document: &str) -> Value {
    let mut value: Value = serde_json::from_str(document).unwrap();
    value["seriess"][0].take()
}

fn observation(value: &str) -> Observation {
    Observation {
        realtime_start: "2013-08-14".into(),
        realtime_end: "2013-08-14".into(),
        date: "1929-01-01".into(),
        value: value.into(),
    }
}

fn observations_page(offset: u32, items: usize, count: u32) -> ObservationsPage {
    let mut page: ObservationsPage = serde_json::from_str(OBSERVATIONS).unwrap();
    page.offset = offset;
    page.count = count;
    page.observations = vec![observation("1.0"); items];
    page
}

fn series_page(offset: u32, items: usize, count: u32) -> SeriesPage {
    let mut page: SeriesPage = serde_json::from_str(SEARCH).unwrap();
    let series = page.series[0].clone();
    page.offset = offset;
    page.count = count;
    page.series = vec![series; items];
    page
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
            seasonal_adjustment: "Not Seasonally Adjusted".into(),
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
    assert_eq!(page.order_by, "observation_date");
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
    assert_eq!(page.series.len(), 1);
    assert_eq!(page.series[0].id, "MSIM2");
    assert_eq!(page.series[0].popularity, Some(34));
    assert_eq!(page.series[0].group_popularity, Some(33));
}

#[test]
fn numeric_value_parses_number_when_value_is_numeric() {
    let observation = observation("1065.9");

    let value = observation.numeric_value();

    assert_eq!(value, Some(1065.9));
}

#[test]
fn numeric_value_returns_none_when_value_is_missing_marker() {
    let observation = observation(".");

    let value = observation.numeric_value();

    assert_eq!(value, None);
}

#[test]
fn numeric_value_returns_none_when_value_is_other_text() {
    let cases = ["", "n/a", "NaN", "inf", "1,000"];

    for text in cases {
        let value = observation(text).numeric_value();

        assert_eq!(value, None, "{text:?}");
    }
}

#[test]
fn next_offset_continues_after_page_when_more_items_remain() {
    let observations = observations_page(100, 50, 200);
    let series = series_page(100, 50, 200);

    let offsets = (observations.next_offset(), series.next_offset());

    assert_eq!(offsets, (Some(150), Some(150)));
}

#[test]
fn next_offset_returns_none_when_page_completes_results() {
    let observations = observations_page(150, 50, 200);
    let series = series_page(150, 50, 200);

    let offsets = (observations.next_offset(), series.next_offset());

    assert_eq!(offsets, (None, None));
}

#[test]
fn next_offset_returns_none_when_page_is_empty() {
    let observations = observations_page(0, 0, 200);
    let series = series_page(0, 0, 200);

    let offsets = (observations.next_offset(), series.next_offset());

    assert_eq!(offsets, (None, None));
}

#[test]
fn next_offset_returns_none_when_offset_would_overflow() {
    let observations = observations_page(u32::MAX - 1, 2, u32::MAX);
    let series = series_page(u32::MAX - 1, 2, u32::MAX);

    let offsets = (observations.next_offset(), series.next_offset());

    assert_eq!(offsets, (None, None));
}

#[test]
fn next_offset_continues_when_offset_is_near_maximum() {
    let observations = observations_page(u32::MAX - 3, 1, u32::MAX);
    let series = series_page(u32::MAX - 3, 1, u32::MAX);

    let offsets = (observations.next_offset(), series.next_offset());

    assert_eq!(offsets, (Some(u32::MAX - 2), Some(u32::MAX - 2)));
}

#[test]
fn serialize_omits_options_when_observations_request_sets_only_series_id() {
    let request = ObservationsRequest {
        series_id: "GNPCA".into(),
        ..Default::default()
    };

    let query = query_pairs(&request);

    assert_eq!(query, pairs(&[("series_id", "GNPCA")]));
}

#[test]
fn serialize_encodes_every_field_when_observations_request_is_populated() {
    let request = ObservationsRequest {
        series_id: "GNPCA".into(),
        observation_start: Some("1990-01-01".into()),
        observation_end: Some("2000-01-01".into()),
        realtime_start: Some("2005-01-01".into()),
        realtime_end: Some("2005-12-31".into()),
        vintage_dates: vec!["2005-01-01".into(), "2005-06-01".into()],
        limit: Some(10),
        offset: Some(20),
        sort_order: Some(SortOrder::Desc),
        units: Some(Units::PercentChangeFromYearAgo),
        frequency: Some(Frequency::Quarterly),
        aggregation_method: Some(AggregationMethod::EndOfPeriod),
    };

    let query = query_pairs(&request);

    assert_eq!(
        query,
        pairs(&[
            ("series_id", "GNPCA"),
            ("observation_start", "1990-01-01"),
            ("observation_end", "2000-01-01"),
            ("realtime_start", "2005-01-01"),
            ("realtime_end", "2005-12-31"),
            ("vintage_dates", "2005-01-01,2005-06-01"),
            ("limit", "10"),
            ("offset", "20"),
            ("sort_order", "desc"),
            ("units", "pc1"),
            ("frequency", "q"),
            ("aggregation_method", "eop"),
        ])
    );
}

#[test]
fn serialize_omits_options_when_search_request_sets_only_search_text() {
    let request = SearchRequest {
        search_text: "money stock".into(),
        ..Default::default()
    };

    let query = query_pairs(&request);

    assert_eq!(query, pairs(&[("search_text", "money stock")]));
}

#[test]
fn serialize_encodes_every_field_when_search_request_is_populated() {
    let request = SearchRequest {
        search_text: "money stock".into(),
        search_type: Some(SearchType::FullText),
        limit: Some(5),
        offset: Some(10),
        order_by: Some(SearchOrderBy::Popularity),
        sort_order: Some(SortOrder::Asc),
        filter_variable: Some(FilterVariable::Frequency),
        filter_value: Some("Monthly".into()),
        tag_names: vec!["usa".into(), "nsa".into()],
        exclude_tag_names: vec!["discontinued".into(), "annual".into()],
        realtime_start: Some("2017-08-01".into()),
        realtime_end: Some("2017-08-02".into()),
    };

    let query = query_pairs(&request);

    assert_eq!(
        query,
        pairs(&[
            ("search_text", "money stock"),
            ("search_type", "full_text"),
            ("limit", "5"),
            ("offset", "10"),
            ("order_by", "popularity"),
            ("sort_order", "asc"),
            ("filter_variable", "frequency"),
            ("filter_value", "Monthly"),
            ("tag_names", "usa;nsa"),
            ("exclude_tag_names", "discontinued;annual"),
            ("realtime_start", "2017-08-01"),
            ("realtime_end", "2017-08-02"),
        ])
    );
}

#[test]
fn serialize_names_sort_orders_when_each_variant_is_encoded() {
    let cases = [(SortOrder::Asc, "asc"), (SortOrder::Desc, "desc")];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}

#[test]
fn serialize_names_units_when_each_variant_is_encoded() {
    let cases = [
        (Units::Levels, "lin"),
        (Units::Change, "chg"),
        (Units::ChangeFromYearAgo, "ch1"),
        (Units::PercentChange, "pch"),
        (Units::PercentChangeFromYearAgo, "pc1"),
        (Units::CompoundedAnnualRateOfChange, "pca"),
        (Units::ContinuouslyCompoundedRateOfChange, "cch"),
        (Units::ContinuouslyCompoundedAnnualRateOfChange, "cca"),
        (Units::NaturalLog, "log"),
    ];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}

#[test]
fn serialize_names_frequencies_when_each_variant_is_encoded() {
    let cases = [
        (Frequency::Daily, "d"),
        (Frequency::Weekly, "w"),
        (Frequency::Biweekly, "bw"),
        (Frequency::Monthly, "m"),
        (Frequency::Quarterly, "q"),
        (Frequency::Semiannual, "sa"),
        (Frequency::Annual, "a"),
        (Frequency::WeeklyEndingFriday, "wef"),
        (Frequency::WeeklyEndingThursday, "weth"),
        (Frequency::WeeklyEndingWednesday, "wew"),
        (Frequency::WeeklyEndingTuesday, "wetu"),
        (Frequency::WeeklyEndingMonday, "wem"),
        (Frequency::WeeklyEndingSunday, "wesu"),
        (Frequency::WeeklyEndingSaturday, "wesa"),
        (Frequency::BiweeklyEndingWednesday, "bwew"),
        (Frequency::BiweeklyEndingMonday, "bwem"),
    ];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}

#[test]
fn serialize_names_aggregation_methods_when_each_variant_is_encoded() {
    let cases = [
        (AggregationMethod::Average, "avg"),
        (AggregationMethod::Sum, "sum"),
        (AggregationMethod::EndOfPeriod, "eop"),
    ];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}

#[test]
fn serialize_names_search_types_when_each_variant_is_encoded() {
    let cases = [
        (SearchType::FullText, "full_text"),
        (SearchType::SeriesId, "series_id"),
    ];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}

#[test]
fn serialize_names_search_orderings_when_each_variant_is_encoded() {
    let cases = [
        (SearchOrderBy::SearchRank, "search_rank"),
        (SearchOrderBy::SeriesId, "series_id"),
        (SearchOrderBy::Title, "title"),
        (SearchOrderBy::Units, "units"),
        (SearchOrderBy::Frequency, "frequency"),
        (SearchOrderBy::SeasonalAdjustment, "seasonal_adjustment"),
        (SearchOrderBy::RealtimeStart, "realtime_start"),
        (SearchOrderBy::RealtimeEnd, "realtime_end"),
        (SearchOrderBy::LastUpdated, "last_updated"),
        (SearchOrderBy::ObservationStart, "observation_start"),
        (SearchOrderBy::ObservationEnd, "observation_end"),
        (SearchOrderBy::Popularity, "popularity"),
        (SearchOrderBy::GroupPopularity, "group_popularity"),
    ];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}

#[test]
fn serialize_names_filter_variables_when_each_variant_is_encoded() {
    let cases = [
        (FilterVariable::Frequency, "frequency"),
        (FilterVariable::Units, "units"),
        (FilterVariable::SeasonalAdjustment, "seasonal_adjustment"),
    ];

    for (variant, expected) in cases {
        let name = wire_name(variant);

        assert_eq!(name, json!(expected), "{variant:?}");
    }
}
