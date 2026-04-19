use log::info;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use crate::notion_client::NotionClient;
use crate::error::HabitsError;

pub fn get_hmd(notion: &NotionClient) -> Result<Vec<(String, String)>, HabitsError> {
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

    let url = format!(
        "{}/databases/{}/query",
        notion.base_url, notion.habits_master_database_id
    );

    let http_client = Client::new();

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

    let habits = parse_habits_response(&result_json)?;

    info!("Loaded {} active habits", habits.len());
    info!("Get Habits Master Data -- End");

    Ok(habits)
}

// Extracted pure function: parses a Notion database query response into
// a vec of (id, name) tuples. No HTTP, no credentials — fully testable.
pub fn parse_habits_response(json: &str) -> Result<Vec<(String, String)>, HabitsError> {
    let v: Value = serde_json::from_str(json)?;

    let results = v["results"]
        .as_array()
        .ok_or(HabitsError::MissingResultsArray)?;

    // `.filter_map()` drops any entries where the name field is missing.
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
        let json = r#"{
            "results": [{
                "id": "habit-uuid-1",
                "properties": {
                    "Name": { "title": [{ "plain_text": "Morning Run" }] }
                }
            }]
        }"#;
        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].0, "habit-uuid-1");
        assert_eq!(habits[0].1, "Morning Run");
    }

    #[test]
    fn test_parse_habits_returns_multiple_habits() {
        let json = r#"{
            "results": [
                { "id": "id-1", "properties": { "Name": { "title": [{ "plain_text": "Habit A" }] } } },
                { "id": "id-2", "properties": { "Name": { "title": [{ "plain_text": "Habit B" }] } } }
            ]
        }"#;
        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert_eq!(habits.len(), 2);
        assert_eq!(habits[0], ("id-1".to_string(), "Habit A".to_string()));
        assert_eq!(habits[1], ("id-2".to_string(), "Habit B".to_string()));
    }

    #[test]
    fn test_parse_habits_skips_entries_without_name() {
        let json = r#"{
            "results": [
                { "id": "id-good", "properties": { "Name": { "title": [{ "plain_text": "Good Habit" }] } } },
                { "id": "id-bad",  "properties": { "Name": { "title": [] } } }
            ]
        }"#;
        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert_eq!(habits.len(), 1);
        assert_eq!(habits[0].1, "Good Habit");
    }

    #[test]
    fn test_parse_habits_returns_empty_vec_on_empty_results() {
        let json = r#"{ "results": [] }"#;
        let habits = parse_habits_response(json).expect("Should parse successfully");
        assert!(habits.is_empty());
    }

    #[test]
    fn test_parse_habits_errors_on_missing_results_key() {
        let json = r#"{ "object": "list" }"#;
        let result = parse_habits_response(json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("results"));
    }

    #[test]
    fn test_parse_habits_errors_on_invalid_json() {
        let result = parse_habits_response("{ bad json");
        assert!(result.is_err());
    }
}
