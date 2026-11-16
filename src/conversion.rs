use std::{error::Error, fmt};

use serde::{
    Deserialize,
    de::{IntoDeserializer, value::Error as ValueError},
};

use crate::market::{Date, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ValueKind {
    Date,
    Timestamp,
    Number,
    Code,
}

impl fmt::Display for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Date => "date",
            Self::Timestamp => "timestamp",
            Self::Number => "number",
            Self::Code => "code",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionError {
    field: &'static str,
    expected: ValueKind,
}

impl ConversionError {
    pub(crate) fn new(field: &'static str, expected: ValueKind) -> Self {
        Self { field, expected }
    }

    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn expected(&self) -> ValueKind {
        self.expected
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Response field `{}` does not hold a valid {}",
            self.field, self.expected
        )
    }
}

impl Error for ConversionError {}

pub(crate) fn date(field: &'static str, value: &str) -> Result<Date, ConversionError> {
    value
        .parse()
        .map_err(|_| ConversionError::new(field, ValueKind::Date))
}

pub(crate) fn timestamp(field: &'static str, value: &str) -> Result<Timestamp, ConversionError> {
    value
        .parse()
        .map_err(|_| ConversionError::new(field, ValueKind::Timestamp))
}

pub(crate) fn number(field: &'static str, value: &str) -> Result<f64, ConversionError> {
    value
        .parse::<f64>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or(ConversionError::new(field, ValueKind::Number))
}

pub(crate) fn known_code<'de, T: Deserialize<'de>>(value: &'de str) -> Option<T> {
    T::deserialize(IntoDeserializer::<ValueError>::into_deserializer(value)).ok()
}

pub(crate) fn code<'de, T: Deserialize<'de>>(
    field: &'static str,
    value: &'de str,
) -> Result<T, ConversionError> {
    known_code(value).ok_or(ConversionError::new(field, ValueKind::Code))
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(rename_all = "lowercase")]
    enum Side {
        Buy,
        Sell,
    }

    #[test]
    fn date_parses_calendar_date_when_value_is_iso_date() {
        let value = "1929-01-01";

        let parsed = date("date", value);

        assert_eq!(parsed, Ok(Date::new(1929, 1, 1).unwrap()));
    }

    #[test]
    fn date_reports_field_when_value_is_not_a_date() {
        let cases = ["", "1929-13-01", "yesterday"];

        for value in cases {
            let parsed = date("date", value);

            assert_eq!(
                parsed,
                Err(ConversionError::new("date", ValueKind::Date)),
                "{value:?}"
            );
        }
    }

    #[test]
    fn timestamp_parses_instant_when_offset_has_hours_only() {
        let value = "2013-07-31 09:26:16-05";

        let parsed = timestamp("last_updated", value);

        assert_eq!(parsed, Ok("2013-07-31T14:26:16Z".parse().unwrap()));
    }

    #[test]
    fn timestamp_reports_field_when_value_has_no_offset() {
        let value = "2013-07-31 09:26:16";

        let parsed = timestamp("last_updated", value);

        assert_eq!(
            parsed,
            Err(ConversionError::new("last_updated", ValueKind::Timestamp))
        );
    }

    #[test]
    fn number_parses_decimal_when_value_is_numeric() {
        let value = "1065.9";

        let parsed = number("value", value);

        assert_eq!(parsed, Ok(1065.9));
    }

    #[test]
    fn number_reports_field_when_value_is_not_a_finite_number() {
        let cases = ["", ".", "n/a", "NaN", "inf", "1,000"];

        for value in cases {
            let parsed = number("value", value);

            assert_eq!(
                parsed,
                Err(ConversionError::new("value", ValueKind::Number)),
                "{value:?}"
            );
        }
    }

    #[test]
    fn code_parses_variant_when_value_is_known() {
        let value = "sell";

        let parsed = code::<Side>("side", value);

        assert_eq!(parsed, Ok(Side::Sell));
    }

    #[test]
    fn code_reports_field_when_value_is_unknown() {
        let value = "short";

        let parsed = code::<Side>("side", value);

        assert_eq!(parsed, Err(ConversionError::new("side", ValueKind::Code)));
    }

    #[test]
    fn known_code_returns_none_when_value_is_unknown() {
        let value = "short";

        let parsed = known_code::<Side>(value);

        assert_eq!(parsed, None);
    }

    #[test]
    fn fmt_names_field_and_kind_when_error_is_displayed() {
        let error = ConversionError::new("observation.value", ValueKind::Number);

        let message = error.to_string();

        assert_eq!(
            message,
            "Response field `observation.value` does not hold a valid number"
        );
    }
}
