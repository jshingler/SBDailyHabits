use log::info;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use crate::notion_client::NotionClient;
use crate::error::HabitsError;

pub fn create_daily_habit(
    notion: &NotionClient,
    habit_id: &str,
    today_id: &str,
    habit_name: &str,
) -> Result<(), HabitsError> {
    info!(
        "create_daily_habit(habit_id: {}, today_id: {}, habit_name: {})",
        habit_id, today_id, habit_name
    );

    let query = build_create_habit_query(
        habit_id,
        today_id,
        habit_name,
        &notion.daily_stats_page_id,
        &notion.habits_database_id,
    );

    let url = format!("{}/pages", notion.base_url);
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

    info!("Response: {}", result_json);
    Ok(())
}

// Idempotency check: returns true if a daily habit entry already exists
// for this habit + day combination. Call this before `create_daily_habit`
// to avoid creating duplicates when the program is run more than once per day.
pub fn habit_exists_today(
    notion: &NotionClient,
    habit_id: &str,
    today_id: &str,
) -> Result<bool, HabitsError> {
    // Notion's relation filter checks whether a relation property contains
    // a specific page ID. Using "and" with both Habit and Day ensures we only
    // match entries that belong to THIS habit on THIS day.
    let query = json!({
        "filter": {
            "and": [
                {
                    "property": "Habit",
                    "relation": { "contains": habit_id }
                },
                {
                    "property": "Day",
                    "relation": { "contains": today_id }
                }
            ]
        }
    });

    let url = format!(
        "{}/databases/{}/query",
        notion.base_url, notion.habits_database_id
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

    parse_habit_exists(&result_json)
}

// Pure function: checks whether the Notion response contains any results.
// A non-empty results array means the entry already exists.
pub fn parse_habit_exists(json: &str) -> Result<bool, HabitsError> {
    let v: serde_json::Value = serde_json::from_str(json)?;
    let results = v["results"]
        .as_array()
        .ok_or(HabitsError::MissingResultsArray)?;
    Ok(!results.is_empty())
}

// Extracted pure function: builds the Notion create-page JSON body.
// Takes all values as parameters so it has no dependency on CONFIG.
pub fn build_create_habit_query(
    habit_id: &str,
    today_id: &str,
    habit_name: &str,
    daily_stats_id: &str,
    habits_database_id: &str,
) -> Value {
    json!({
        "parent": {
            "database_id": habits_database_id
        },
        "properties": {
            "Name": {
                "title": [{ "text": { "content": habit_name } }]
            },
            "Habit": {
                "relation": [{ "id": habit_id }]
            },
            "Day": {
                "relation": [{ "id": today_id }]
            },
            "stats": {
                "relation": [{ "id": daily_stats_id }]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_query() -> Value {
        build_create_habit_query(
            "habit-id-123",
            "today-id-456",
            "Morning Run",
            "stats-id-789",
            "habits-db-id-abc",
        )
    }

    #[test]
    fn test_query_contains_habit_name_in_title() {
        let query = make_query();
        let content = query["properties"]["Name"]["title"][0]["text"]["content"]
            .as_str().expect("title content should be a string");
        assert_eq!(content, "Morning Run");
    }

    #[test]
    fn test_query_contains_correct_habit_relation() {
        let query = make_query();
        let id = query["properties"]["Habit"]["relation"][0]["id"]
            .as_str().expect("Habit relation id should be a string");
        assert_eq!(id, "habit-id-123");
    }

    #[test]
    fn test_query_contains_correct_day_relation() {
        let query = make_query();
        let id = query["properties"]["Day"]["relation"][0]["id"]
            .as_str().expect("Day relation id should be a string");
        assert_eq!(id, "today-id-456");
    }

    #[test]
    fn test_query_contains_correct_stats_relation() {
        let query = make_query();
        let id = query["properties"]["stats"]["relation"][0]["id"]
            .as_str().expect("stats relation id should be a string");
        assert_eq!(id, "stats-id-789");
    }

    #[test]
    fn test_query_contains_correct_parent_database() {
        let query = make_query();
        let db_id = query["parent"]["database_id"]
            .as_str().expect("parent database_id should be a string");
        assert_eq!(db_id, "habits-db-id-abc");
    }

    // ── Idempotency: parse_habit_exists ───────────────────────────────────────

    #[test]
    fn test_parse_habit_exists_returns_true_when_results_non_empty() {
        // Verifies that when Notion finds an existing entry for this habit+day,
        // we correctly detect it and return true (meaning: skip creation).
        let json = r#"{ "results": [{ "id": "existing-entry-id" }] }"#;
        let exists = parse_habit_exists(json).expect("Should parse");
        assert!(exists, "Should return true when an existing entry is found");
    }

    #[test]
    fn test_parse_habit_exists_returns_false_when_results_empty() {
        // Verifies that when Notion finds no existing entry, we return false
        // (meaning: safe to create a new one).
        let json = r#"{ "results": [] }"#;
        let exists = parse_habit_exists(json).expect("Should parse");
        assert!(!exists, "Should return false when no existing entry found");
    }

    #[test]
    fn test_parse_habit_exists_errors_on_invalid_json() {
        // Verifies that malformed JSON produces an error, not a panic.
        let result = parse_habit_exists("{ not valid json");
        assert!(result.is_err());
    }
}
