use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    #[serde(rename = "lin")]
    Levels,
    #[serde(rename = "chg")]
    Change,
    #[serde(rename = "ch1")]
    ChangeFromYearAgo,
    #[serde(rename = "pch")]
    PercentChange,
    #[serde(rename = "pc1")]
    PercentChangeFromYearAgo,
    #[serde(rename = "pca")]
    CompoundedAnnualRateOfChange,
    #[serde(rename = "cch")]
    ContinuouslyCompoundedRateOfChange,
    #[serde(rename = "cca")]
    ContinuouslyCompoundedAnnualRateOfChange,
    #[serde(rename = "log")]
    NaturalLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Frequency {
    #[serde(rename = "d")]
    Daily,
    #[serde(rename = "w")]
    Weekly,
    #[serde(rename = "bw")]
    Biweekly,
    #[serde(rename = "m")]
    Monthly,
    #[serde(rename = "q")]
    Quarterly,
    #[serde(rename = "sa")]
    Semiannual,
    #[serde(rename = "a")]
    Annual,
    #[serde(rename = "wef")]
    WeeklyEndingFriday,
    #[serde(rename = "weth")]
    WeeklyEndingThursday,
    #[serde(rename = "wew")]
    WeeklyEndingWednesday,
    #[serde(rename = "wetu")]
    WeeklyEndingTuesday,
    #[serde(rename = "wem")]
    WeeklyEndingMonday,
    #[serde(rename = "wesu")]
    WeeklyEndingSunday,
    #[serde(rename = "wesa")]
    WeeklyEndingSaturday,
    #[serde(rename = "bwew")]
    BiweeklyEndingWednesday,
    #[serde(rename = "bwem")]
    BiweeklyEndingMonday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AggregationMethod {
    #[serde(rename = "avg")]
    Average,
    #[serde(rename = "sum")]
    Sum,
    #[serde(rename = "eop")]
    EndOfPeriod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    FullText,
    SeriesId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchOrderBy {
    SearchRank,
    SeriesId,
    Title,
    Units,
    Frequency,
    SeasonalAdjustment,
    RealtimeStart,
    RealtimeEnd,
    LastUpdated,
    ObservationStart,
    ObservationEnd,
    Popularity,
    GroupPopularity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterVariable {
    Frequency,
    Units,
    SeasonalAdjustment,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ObservationsRequest {
    pub series_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realtime_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realtime_end: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", serialize_with = "comma_joined")]
    pub vintage_dates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub units: Option<Units>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Frequency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregation_method: Option<AggregationMethod>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SearchRequest {
    pub search_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_type: Option<SearchType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<SearchOrderBy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<SortOrder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_variable: Option<FilterVariable>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_value: Option<String>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "semicolon_joined"
    )]
    pub tag_names: Vec<String>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "semicolon_joined"
    )]
    pub exclude_tag_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realtime_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realtime_end: Option<String>,
}

fn comma_joined<S: Serializer>(values: &[String], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&values.join(","))
}

fn semicolon_joined<S: Serializer>(values: &[String], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&values.join(";"))
}
