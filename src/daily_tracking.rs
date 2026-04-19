use tracing::info;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use chrono::Local;
use crate::notion_client::NotionClient;
use crate::error::HabitsError;

// Accepts a `NotionClient` so callers can inject test credentials pointing
// at a mock server instead of the real Notion API.
pub fn get_today_id(notion: &NotionClient) -> Result<String, HabitsError> {
    // `Local::now().format()` is cleaner than manual year/month/day formatting.
    // The format string follows strftime conventions.
    let formatted_date = Local::now().format("%Y-%m-%d").to_string();

    let query = json!({
        "filter": {
            "and": [
                {
                    "timestamp": "created_time",
                    "created_time": {
                        "equals": formatted_date
                    }
                }
            ]
        },
        "sorts": [
            {
                "property": "Date",
                "direction": "descending"
            }
        ]
    });

    let url = format!(
        "{}/databases/{}/query",
        notion.base_url, notion.daily_database_id
    );

    let http_client = Client::new();

    // The `?` operator propagates errors up to the caller immediately.
    let response = http_client
        .post(&url)
        .header("Authorization", &notion.token)
        .header("Notion-Version", &notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()?;

    let status = response.status();
    let result_json = response.text()?;

    if !status.is_success() {
        return Err(HabitsError::NotionApi {
            status: status.as_u16(),
            body: result_json,
        });
    }

    parse_today_id(&result_json)
}

// Extracted pure function: takes a JSON string, returns the first page ID.
// Keeping HTTP and parsing separate makes each independently testable.
pub fn parse_today_id(json: &str) -> Result<String, HabitsError> {
    let parsed: ApiResponse = serde_json::from_str(json)?;

    // `.first()` returns `Option<&T>` — either `Some(&item)` or `None`.
    // This is safer than `[0]` which panics on an empty Vec.
    let today_id = parsed
        .results
        .first()
        .ok_or(HabitsError::NoDayEntry)?
        .id
        .clone();

    info!(page_id = %today_id, "Found today's Notion page");
    Ok(today_id)
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    results: Vec<Page>,
}

#[derive(Deserialize, Debug)]
struct Page {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_today_id_returns_first_result() {
        let json = r#"{
            "object": "list",
            "results": [
                { "id": "abc-123", "object": "page" }
            ]
        }"#;
        let id = parse_today_id(json).expect("Should parse successfully");
        assert_eq!(id, "abc-123");
    }

    #[test]
    fn test_parse_today_id_returns_first_when_multiple_results() {
        let json = r#"{
            "results": [
                { "id": "first-id" },
                { "id": "second-id" }
            ]
        }"#;
        let id = parse_today_id(json).expect("Should parse successfully");
        assert_eq!(id, "first-id");
    }

    #[test]
    fn test_parse_today_id_errors_on_empty_results() {
        let json = r#"{ "results": [] }"#;
        let result = parse_today_id(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No day entry"), "got: {}", err);
    }

    #[test]
    fn test_parse_today_id_errors_on_invalid_json() {
        let result = parse_today_id("not valid json {{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_date_format_shape() {
        let formatted = Local::now().format("%Y-%m-%d").to_string();
        let parts: Vec<&str> = formatted.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 4, "Year should be 4 digits");
        assert_eq!(parts[1].len(), 2, "Month should be 2 digits");
        assert_eq!(parts[2].len(), 2, "Day should be 2 digits");
    }
}
