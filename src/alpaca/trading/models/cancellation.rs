use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResult {
    pub id: String,
    pub status: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn deserialize_preserves_each_status_when_cancellations_have_mixed_results() {
        let response = json!([
            {"id": "first-order", "status": 200},
            {"id": "second-order", "status": 500}
        ]);

        let results: Vec<CancelOrderResult> = serde_json::from_value(response).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "first-order");
        assert_eq!(results[0].status, 200);
        assert_eq!(results[1].id, "second-order");
        assert_eq!(results[1].status, 500);
    }

    #[test]
    fn deserialize_rejects_result_when_status_is_missing() {
        let response = json!({"id": "first-order"});

        let result = serde_json::from_value::<CancelOrderResult>(response);

        assert!(result.is_err());
    }
}
