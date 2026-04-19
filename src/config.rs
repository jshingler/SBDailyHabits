use serde::Deserialize;
use serde_java_properties::from_reader;
use std::fs::File;
use std::io::Read;
use crate::error::HabitsError;

// `once_cell::Lazy` lets us initialize a value exactly once, the first time it's
// accessed, and then reuse it everywhere. This is Rust's answer to a global
// singleton — CONFIG is loaded from disk once and shared across all modules.
use once_cell::sync::Lazy;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    #[serde(rename = "app.name")]
    pub name: String,

    #[serde(rename = "app.version")]
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "database.user")]
    pub user: String,

    #[serde(rename = "database.password")]
    pub password: String,

    #[serde(rename = "database.host")]
    pub host: String,

    #[serde(rename = "database.port")]
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct NotionApiConfig {
    #[serde(rename = "notion.api.url")]
    pub url: String,

    #[serde(rename = "notion.api.version")]
    pub version: String,

    #[serde(rename = "notion.api.token")]
    pub token: String,

    #[serde(rename = "daily.database.id")]
    pub daily_database_id: String,

    #[serde(rename = "habits.database.id")]
    pub habits_database_id: String,

    #[serde(rename = "habits.master.database.id")]
    pub habits_master_database_id: String,

    #[serde(rename = "daily.stats.page.id")]
    pub daily_stats_page_id: String,
}

// `#[serde(flatten)]` merges the fields of a nested struct into the parent
// during deserialization. This lets us split config into logical groups
// (AppConfig, DatabaseConfig, etc.) while reading from a flat .properties file.
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub app: AppConfig,

    #[serde(flatten)]
    pub database: DatabaseConfig,

    #[serde(flatten)]
    pub notion: NotionApiConfig,
}

// Accepts any `Read` impl (File, Cursor<&[u8]>, etc.) so this function is
// testable without touching the filesystem.
pub fn parse_config<R: Read>(reader: R) -> Result<Config, HabitsError> {
    let config: Config = from_reader(reader)
        .map_err(|e| HabitsError::Config(e.to_string()))?;
    Ok(config)
}

fn load_config() -> Result<Config, HabitsError> {
    let file = File::open("config/config.properties")
        .map_err(|e| HabitsError::Config(e.to_string()))?;
    parse_config(file)
}

// `Lazy::new` wraps a closure. The closure runs once on first access.
// `.expect()` is like `.unwrap()` but lets you provide a message — use it
// for failures that are truly unrecoverable (missing config at startup).
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    load_config().expect("Failed to load config/config.properties")
});

// `#[cfg(test)]` means this module is compiled only when running `cargo test`.
// It's Rust's built-in way to colocate tests with the code they test.
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper that builds a minimal valid properties string for tests.
    fn valid_properties() -> &'static str {
        "app.name=TestApp\n\
         app.version=1.0.0\n\
         database.user=user\n\
         database.password=pass\n\
         database.host=localhost\n\
         database.port=5432\n\
         notion.api.url=https://api.notion.com/v1\n\
         notion.api.version=2022-06-28\n\
         notion.api.token=secret_abc\n\
         daily.database.id=daily-db-id\n\
         habits.database.id=habits-db-id\n\
         habits.master.database.id=master-db-id\n\
         daily.stats.page.id=stats-page-id\n"
    }

    #[test]
    fn test_config_parses_valid_properties() {
        // Verifies that all config fields are correctly deserialized from
        // a properties string — the happy path for config loading.
        let cursor = Cursor::new(valid_properties().as_bytes());
        let config = parse_config(cursor).expect("Should parse valid properties");

        assert_eq!(config.app.name, "TestApp");
        assert_eq!(config.app.version, "1.0.0");
        assert_eq!(config.database.user, "user");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.notion.url, "https://api.notion.com/v1");
        assert_eq!(config.notion.token, "secret_abc");
        assert_eq!(config.notion.habits_master_database_id, "master-db-id");
        assert_eq!(config.notion.daily_stats_page_id, "stats-page-id");
    }

    #[test]
    fn test_config_parse_fails_on_empty_input() {
        // Verifies that an empty reader produces an error rather than a
        // silently broken Config with empty strings.
        let cursor = Cursor::new(b"" as &[u8]);
        let result = parse_config(cursor);
        assert!(result.is_err(), "Empty input should fail to parse");
    }
}
