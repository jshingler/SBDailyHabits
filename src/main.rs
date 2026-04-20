use sb_daily_habits::config;
use sb_daily_habits::error::HabitsError;
use sb_daily_habits::notion_client::NotionClient;
use sb_daily_habits::daily_tracking;
use sb_daily_habits::habits_md;
use sb_daily_habits::daily_habits::{self, get_existing_habit_ids_today};

// `tracing` is a structured logging framework. Unlike the `log` crate which
// only accepts string messages, tracing lets you attach typed key=value fields
// to each log event. Those fields can be queried, filtered, and exported to
// observability tools (Datadog, Jaeger, etc.) without parsing strings.
use tracing::info;

type Result<T> = std::result::Result<T, HabitsError>;

fn main() -> Result<()> {
    // `tracing_subscriber::fmt()` sets up a human-readable console subscriber.
    // It reads the RUST_LOG environment variable to control the log level, e.g.:
    //   RUST_LOG=info cargo run   → shows info and above
    //   RUST_LOG=debug cargo run  → shows everything
    // This replaces the log4rs.yaml file — no external config file needed.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Structured fields use `field = %value` syntax.
    // `%` formats using Display (like {}), `?` uses Debug (like {:?}).
    // The message string comes last — fields are searchable metadata.
    info!(
        app_name = %config::CONFIG.app_name,
        app_version = %config::CONFIG.app_version,
        "Starting SBDailyHabits"
    );

    // Build a NotionClient from the global config. In production this always
    // comes from CONFIG; in tests a mock client pointing at a local server is used.
    let notion = NotionClient::new(
        &config::CONFIG.notion_url,
        &config::CONFIG.notion_token,
        &config::CONFIG.notion_version,
        &config::CONFIG.daily_database_id,
        &config::CONFIG.habits_master_database_id,
        &config::CONFIG.habits_database_id,
        &config::CONFIG.daily_stats_page_id,
    );

    let todays_id = daily_tracking::get_today_id(&notion)?;
    let habits = habits_md::get_hmd(&notion)?;

    // Fetch all existing habit entries for today in a single API call, then
    // check membership in memory. This replaces one Notion query per habit
    // (O(N) API calls) with one batch query regardless of how many habits exist.
    let existing_ids = get_existing_habit_ids_today(&notion, &todays_id)?;
    info!(count = existing_ids.len(), "Loaded existing habit entries for today");

    for (habit_id, habit_name) in habits {
        if existing_ids.contains(&habit_id) {
            info!(habit = %habit_name, "Skipping — entry already exists for today");
        } else if let Err(e) = daily_habits::create_daily_habit(&notion, &habit_id, &todays_id, &habit_name) {
            eprintln!("Failed to create habit '{}': {}", habit_name, e);
        }
    }

    Ok(())
}
