use serde::Deserialize;
use once_cell::sync::Lazy;
use crate::error::HabitsError;

// `dotenvy` loads a `.env` file into the process environment at startup.
// `envy` then deserializes those environment variables into a typed struct
// using serde. This is the idiomatic Rust pattern for configuration:
//
//   .env file → process environment → envy → typed Config struct
//
// `envy` automatically maps SCREAMING_SNAKE_CASE env var names to lowercase
// snake_case Rust field names — no `#[serde(rename)]` needed. For example,
// the env var `NOTION_TOKEN` maps to the Rust field `notion_token`.
//
// `envy` does NOT support `#[serde(flatten)]`, so all fields live directly
// in Config rather than in nested sub-structs.
//
// In production/CI, you skip the .env file and set env vars directly —
// dotenvy silently does nothing if .env doesn't exist, which is correct.

#[derive(Debug, Deserialize)]
pub struct Config {
    pub app_name: String,
    pub app_version: String,
    pub database_user: String,
    pub database_password: String,
    pub database_host: String,
    pub database_port: u16,
    pub notion_url: String,
    pub notion_version: String,
    pub notion_token: String,
    pub daily_database_id: String,
    pub habits_database_id: String,
    pub habits_master_database_id: String,
    pub daily_stats_page_id: String,
}

// Reads config from the current process environment.
// Separated from load_config so tests can set env vars then call this directly.
pub fn parse_config() -> Result<Config, HabitsError> {
    envy::from_env::<Config>().map_err(|e| HabitsError::Config(e.to_string()))
}

fn load_config() -> Result<Config, HabitsError> {
    // `dotenv().ok()` loads .env if it exists, ignores error if it doesn't.
    // This means the same binary works locally (reads .env) and in CI
    // (reads env vars set by the CI system) without any code changes.
    dotenvy::dotenv().ok();
    parse_config()
}

// `Lazy::new` wraps a closure. The closure runs once on first access.
// `.expect()` is appropriate here — missing config at startup is unrecoverable.
pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    load_config().expect("Failed to load config from environment. Is .env present?")
});

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: builds the full set of env vars needed for a valid Config.
    // Returns a vec of (key, Option<value>) pairs for use with temp_env::with_vars.
    fn valid_env_vars() -> Vec<(&'static str, Option<&'static str>)> {
        vec![
            ("APP_NAME",                  Some("TestApp")),
            ("APP_VERSION",               Some("1.0.0")),
            ("DATABASE_USER",             Some("user")),
            ("DATABASE_PASSWORD",         Some("pass")),
            ("DATABASE_HOST",             Some("localhost")),
            ("DATABASE_PORT",             Some("5432")),
            ("NOTION_URL",                Some("https://api.notion.com/v1")),
            ("NOTION_VERSION",            Some("2022-06-28")),
            ("NOTION_TOKEN",              Some("secret_abc")),
            ("DAILY_DATABASE_ID",         Some("daily-db-id")),
            ("HABITS_DATABASE_ID",        Some("habits-db-id")),
            ("HABITS_MASTER_DATABASE_ID", Some("master-db-id")),
            ("DAILY_STATS_PAGE_ID",       Some("stats-page-id")),
        ]
    }

    #[test]
    fn test_config_parses_valid_env_vars() {
        // `temp_env::with_vars` sets env vars for the duration of the closure,
        // then restores the originals — safe to run in parallel tests.
        temp_env::with_vars(valid_env_vars(), || {
            let config = parse_config().expect("Should parse with all vars set");

            assert_eq!(config.app_name, "TestApp");
            assert_eq!(config.app_version, "1.0.0");
            assert_eq!(config.database_user, "user");
            assert_eq!(config.database_port, 5432);
            assert_eq!(config.notion_url, "https://api.notion.com/v1");
            assert_eq!(config.notion_token, "secret_abc");
            assert_eq!(config.habits_master_database_id, "master-db-id");
            assert_eq!(config.daily_stats_page_id, "stats-page-id");
        });
    }

    #[test]
    fn test_config_fails_when_required_var_missing() {
        // Verifies that a missing required env var produces an error.
        let vars_without_token: Vec<(&str, Option<&str>)> = valid_env_vars()
            .into_iter()
            .map(|(k, v)| if k == "NOTION_TOKEN" { (k, None) } else { (k, v) })
            .collect();

        temp_env::with_vars(vars_without_token, || {
            let result = parse_config();
            assert!(result.is_err(), "Should fail when NOTION_TOKEN is missing");
        });
    }
}
