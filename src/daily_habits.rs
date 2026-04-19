use log::info;
use reqwest::blocking::Client;
use serde_json::{json, Value};
use crate::config::CONFIG;

// `&str` is the idiomatic borrowed string type for function parameters.
// Prefer `&str` over `&String` — it's more flexible: it accepts both `String`
// references and string literals (&str), while `&String` only accepts `String`.
pub fn create_daily_habit(
    habit_id: &str,
    today_id: &str,
    habit_name: &str,
    daily_stats_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!(
        "create_daily_habit(habit_id: {}, today_id: {}, habit_name: {}, daily_stats_id: {})",
        habit_id, today_id, habit_name, daily_stats_id
    );

    let query = build_create_habit_query(
        habit_id,
        today_id,
        habit_name,
        daily_stats_id,
        &CONFIG.notion.habits_database_id,
    );

    let notion_database_url = format!("{}/pages", CONFIG.notion.url);

    let http_client = Client::new();

    // The `?` operator propagates any error from send() up to the caller.
    let response = http_client
        .post(notion_database_url)
        .header("Authorization", &CONFIG.notion.token)
        .header("Notion-Version", &CONFIG.notion.version)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()?;

    let result_json = response.text()?;
    info!("Response: {}", result_json);

    Ok(())
}

// Extracted pure function: builds the JSON body for creating a Notion page.
// Takes the database ID as a parameter so it doesn't depend on CONFIG,
// making it testable without a config file.
pub(crate) fn build_create_habit_query(
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
                "title": [
                    {
                        "text": {
                            "content": habit_name
                        }
                    }
                ]
            },
            "Habit": {
                "relation": [
                    { "id": habit_id }
                ]
            },
            "Day": {
                "relation": [
                    { "id": today_id }
                ]
            },
            "stats": {
                "relation": [
                    { "id": daily_stats_id }
                ]
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to call build_create_habit_query with fixed test values.
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
        // Verifies the habit name is placed in the Notion title property,
        // which is what makes the page show up with the right name in Notion.
        let query = make_query();
        let content = query["properties"]["Name"]["title"][0]["text"]["content"]
            .as_str()
            .expect("title content should be a string");
        assert_eq!(content, "Morning Run");
    }

    #[test]
    fn test_query_contains_correct_habit_relation() {
        // Verifies the Habit relation links to the correct habit ID from
        // the Habits Master database.
        let query = make_query();
        let id = query["properties"]["Habit"]["relation"][0]["id"]
            .as_str()
            .expect("Habit relation id should be a string");
        assert_eq!(id, "habit-id-123");
    }

    #[test]
    fn test_query_contains_correct_day_relation() {
        // Verifies the Day relation links to today's Notion page ID,
        // connecting this habit entry to the correct day.
        let query = make_query();
        let id = query["properties"]["Day"]["relation"][0]["id"]
            .as_str()
            .expect("Day relation id should be a string");
        assert_eq!(id, "today-id-456");
    }

    #[test]
    fn test_query_contains_correct_stats_relation() {
        // Verifies the stats relation links to the daily stats page,
        // which drives the rollup statistics in Notion.
        let query = make_query();
        let id = query["properties"]["stats"]["relation"][0]["id"]
            .as_str()
            .expect("stats relation id should be a string");
        assert_eq!(id, "stats-id-789");
    }

    #[test]
    fn test_query_contains_correct_parent_database() {
        // Verifies the parent database_id points to the Daily Habits database,
        // not the Habits Master or Daily Tracking database.
        let query = make_query();
        let db_id = query["parent"]["database_id"]
            .as_str()
            .expect("parent database_id should be a string");
        assert_eq!(db_id, "habits-db-id-abc");
    }
}
