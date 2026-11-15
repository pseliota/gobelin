use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Clock {
    pub timestamp: String,
    pub is_open: bool,
    pub next_open: String,
    pub next_close: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarDay {
    pub date: String,
    pub open: String,
    pub close: String,
    pub session_open: Option<String>,
    pub session_close: Option<String>,
    pub settlement_date: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CalendarRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_type: Option<CalendarDateType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalendarDateType {
    Trading,
    Settlement,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_preserves_clock_fields_when_market_is_closed() {
        let response = json!({
            "timestamp": "2025-06-24T18:15:22-04:00",
            "is_open": false,
            "next_open": "2025-06-25T09:30:00-04:00",
            "next_close": "2025-06-25T16:00:00-04:00"
        });

        let clock: Clock = serde_json::from_value(response).unwrap();

        assert!(!clock.is_open);
        assert_eq!(clock.timestamp, "2025-06-24T18:15:22-04:00");
        assert_eq!(clock.next_open, "2025-06-25T09:30:00-04:00");
        assert_eq!(clock.next_close, "2025-06-25T16:00:00-04:00");
    }

    #[test]
    fn deserialize_rejects_clock_when_open_flag_is_missing() {
        let response = json!({
            "timestamp": "2025-06-24T18:15:22-04:00",
            "next_open": "2025-06-25T09:30:00-04:00",
            "next_close": "2025-06-25T16:00:00-04:00"
        });

        let result = serde_json::from_value::<Clock>(response);

        assert!(result.is_err());
    }

    #[test]
    fn deserialize_preserves_session_formats_when_calendar_has_early_close() {
        let response = json!({
            "date": "2025-11-28", "open": "09:30", "close": "13:00",
            "session_open": "0400", "session_close": "1700",
            "settlement_date": "2025-12-01"
        });

        let day: CalendarDay = serde_json::from_value(response).unwrap();

        assert_eq!(day.open, "09:30");
        assert_eq!(day.close, "13:00");
        assert_eq!(day.session_open.as_deref(), Some("0400"));
        assert_eq!(day.session_close.as_deref(), Some("1700"));
        assert_eq!(day.settlement_date.as_deref(), Some("2025-12-01"));
    }

    #[test]
    fn deserialize_leaves_session_unknown_when_legacy_calendar_omits_fields() {
        let response = json!({"date": "2025-01-02", "open": "09:30", "close": "16:00"});

        let day: CalendarDay = serde_json::from_value(response).unwrap();

        assert_eq!(day.session_open, None);
        assert_eq!(day.session_close, None);
        assert_eq!(day.settlement_date, None);
    }

    #[test]
    fn serialize_omits_filters_when_calendar_request_is_default() {
        let request = CalendarRequest::default();

        let body = serde_json::to_value(request).unwrap();

        assert_eq!(body, json!({}));
    }

    #[test]
    fn serialize_encodes_query_when_settlement_dates_are_requested() {
        let query = CalendarRequest {
            start: Some("2025-01-02".into()),
            end: Some("2025-01-03".into()),
            date_type: Some(CalendarDateType::Settlement),
        };

        let request = reqwest::blocking::Client::new()
            .get("http://localhost/v2/calendar")
            .query(&query)
            .build()
            .unwrap();
        let pairs: std::collections::HashMap<_, _> =
            request.url().query_pairs().into_owned().collect();

        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs["start"], "2025-01-02");
        assert_eq!(pairs["end"], "2025-01-03");
        assert_eq!(pairs["date_type"], "SETTLEMENT");
    }
}
