use log::info;
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use chrono::Local;
use crate::config::CONFIG;

pub fn get_today_id() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

    let notion_database_url = format!(
        "{}/databases/{}/query",
        CONFIG.notion.url,
        CONFIG.notion.daily_database_id
    );

    let http_client = Client::new();

    // The `?` operator is the idiomatic way to propagate errors in Rust.
    // It means: "if this is Err, return that error from this function immediately."
    let response = http_client
        .post(notion_database_url)
        .header("Authorization", &CONFIG.notion.token)
        .header("Notion-Version", &CONFIG.notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()?;

    let result_json = response.text()?;
    parse_today_id(&result_json)
}

// Extracted pure function: takes a JSON string, returns the first page ID.
// Keeping HTTP and parsing separate makes each independently testable.
pub(crate) fn parse_today_id(
    json: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let parsed: ApiResponse = serde_json::from_str(json)?;

    // `.first()` returns `Option<&T>` — either `Some(&item)` or `None`.
    // This is safer than `[0]` which panics on an empty Vec.
    // We use `ok_or` to convert `None` into a descriptive error.
    let today_id = parsed
        .results
        .first()
        .ok_or("No day entry found in Notion for today. Did you create today's page?")?
        .id
        .clone();

    info!("Today's Notion page ID: {}", today_id);
    Ok(today_id)
}

// Local structs for deserializing just the fields we need from the Notion response.
// Using a typed struct is safer than indexing into raw JSON.
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
        // Verifies the happy path: a valid Notion response with one result
        // returns the ID of that page.
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
        // Verifies that when Notion returns multiple pages, we take the first
        // one (the query sorts descending by date, so first = most recent).
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
        // Verifies the error path: if Notion returns no pages for today,
        // we get a meaningful error instead of an index panic.
        let json = r#"{ "results": [] }"#;

        let result = parse_today_id(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("No day entry"),
            "Error should mention missing day entry, got: {}",
            err
        );
    }

    #[test]
    fn test_parse_today_id_errors_on_invalid_json() {
        // Verifies that malformed JSON produces an error, not a panic.
        let result = parse_today_id("not valid json {{{");
        assert!(result.is_err());
    }

    #[test]
    fn test_date_format_shape() {
        // Verifies the date format produces a YYYY-MM-DD shaped string.
        // We can't assert the exact date, but we can assert the format.
        let formatted = Local::now().format("%Y-%m-%d").to_string();
        let parts: Vec<&str> = formatted.split('-').collect();

        assert_eq!(parts.len(), 3, "Date should have 3 parts separated by '-'");
        assert_eq!(parts[0].len(), 4, "Year should be 4 digits");
        assert_eq!(parts[1].len(), 2, "Month should be 2 digits");
        assert_eq!(parts[2].len(), 2, "Day should be 2 digits");
    }
}
