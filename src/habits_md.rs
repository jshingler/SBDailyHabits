use log::info;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use crate::config::CONFIG;

pub fn get_hmd() -> Result<Vec<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
    info!("Get Habits Master Data -- Start");

    let query = json!({
        "filter": {
            "and": [
                {
                    "property": "Status",
                    "select": {
                        "equals": "Active"
                    }
                }
            ]
        },
        "sorts": [
            {
                "property": "Name",
                "direction": "ascending"
            }
        ]
    });

    let notion_database_url = format!(
        "{}/databases/{}/query",
        CONFIG.notion.url,
        CONFIG.notion.habits_master_database_id
    );

    let http_client = Client::new();

    let response = http_client
        .post(notion_database_url)
        .header("Authorization", &CONFIG.notion.token)
        .header("Notion-Version", &CONFIG.notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()?;

    let result_json = response.text()?;
    let habits = parse_habits_response(&result_json)?;

    info!("Loaded {} active habits", habits.len());
    info!("Get Habits Master Data -- End");

    Ok(habits)
}

// Extracted pure function: parses a Notion database query response into
// a vec of (id, name) tuples. No HTTP, no CONFIG — fully testable.
pub(crate) fn parse_habits_response(
    json: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error + Send + Sync>> {
    // Deserializing into a generic `Value` lets us navigate the JSON structure
    // without defining a full struct for every field.
    let v: Value = serde_json::from_str(json)?;

    // `as_array()` returns `Option<&Vec<Value>>`. We use `ok_or` to turn
    // `None` into an error rather than panicking with `.unwrap()`.
    let results = v["results"]
        .as_array()
        .ok_or("Notion response missing 'results' array")?;

    // `.filter_map()` is like `.map()` but drops any `None` values.
    // This lets us skip results where the name is missing rather than failing.
    let ids_and_names: Vec<(String, String)> = results
        .iter()
        .filter_map(|result| {
            let id = result["id"].as_str()?.to_string();
            let name = result["properties"]["Name"]["title"][0]["plain_text"]
                .as_str()?
                .to_string();
            Some((id, name))
        })
        .collect();

    Ok(ids_and_names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_habits_returns_id_and_name() {
        // Verifies the happy path: a valid Notion response with one active habit
        // returns a vec containing that habit's id and name.
        let json = r#"{
            "results": [
                {
                    "id": "habit-uuid-1",
                    "properties": {
                        "Name": {
                            "title": [
                                { "plain_text": "Morning Run" }
                            ]
                        }
                    }
                }
            ]
        }"#;

        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].0, "habit-uuid-1");
        assert_eq!(habits[0].1, "Morning Run");
    }

    #[test]
    fn test_parse_habits_returns_multiple_habits() {
        // Verifies that multiple habits are all collected, in order.
        let json = r#"{
            "results": [
                {
                    "id": "id-1",
                    "properties": { "Name": { "title": [{ "plain_text": "Habit A" }] } }
                },
                {
                    "id": "id-2",
                    "properties": { "Name": { "title": [{ "plain_text": "Habit B" }] } }
                }
            ]
        }"#;

        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert_eq!(habits.len(), 2);
        assert_eq!(habits[0], ("id-1".to_string(), "Habit A".to_string()));
        assert_eq!(habits[1], ("id-2".to_string(), "Habit B".to_string()));
    }

    #[test]
    fn test_parse_habits_skips_entries_without_name() {
        // Verifies that filter_map silently drops entries with a missing or
        // malformed Name field rather than failing the whole response.
        let json = r#"{
            "results": [
                {
                    "id": "id-good",
                    "properties": { "Name": { "title": [{ "plain_text": "Good Habit" }] } }
                },
                {
                    "id": "id-bad",
                    "properties": { "Name": { "title": [] } }
                }
            ]
        }"#;

        let habits = parse_habits_response(json).expect("Should parse successfully");
        // Only the entry with a valid name survives filter_map
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].1, "Good Habit");
    }

    #[test]
    fn test_parse_habits_returns_empty_vec_on_empty_results() {
        // Verifies that an empty results array is valid — returns empty vec,
        // not an error.
        let json = r#"{ "results": [] }"#;

        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert!(habits.is_empty());
    }

    #[test]
    fn test_parse_habits_errors_on_missing_results_key() {
        // Verifies that a response without a "results" key returns an error,
        // not a panic.
        let json = r#"{ "object": "list" }"#;

        let result = parse_habits_response(json);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("results"),
            "Error should mention 'results', got: {}",
            err
        );
    }

    #[test]
    fn test_parse_habits_errors_on_invalid_json() {
        // Verifies that malformed JSON returns an error, not a panic.
        let result = parse_habits_response("{ bad json");
        assert!(result.is_err());
    }
}
