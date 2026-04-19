use sb_daily_habits::config;
use sb_daily_habits::error::HabitsError;
use sb_daily_habits::notion_client::NotionClient;
use sb_daily_habits::daily_tracking;
use sb_daily_habits::habits_md;
use sb_daily_habits::daily_habits::{self, habit_exists_today};

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
        app_name = %config::CONFIG.app.name,
        app_version = %config::CONFIG.app.version,
        "Starting SBDailyHabits"
    );

    // Build a NotionClient from the global config. In production this always
    // comes from CONFIG; in tests a mock client pointing at a local server is used.
    let notion = NotionClient::new(
        &config::CONFIG.notion.url,
        &config::CONFIG.notion.token,
        &config::CONFIG.notion.version,
        &config::CONFIG.notion.daily_database_id,
        &config::CONFIG.notion.habits_master_database_id,
        &config::CONFIG.notion.habits_database_id,
        &config::CONFIG.notion.daily_stats_page_id,
    );

    let todays_id = daily_tracking::get_today_id(&notion)?;
    let habits = habits_md::get_hmd(&notion)?;

    for (habit_id, habit_name) in habits {
        // Idempotency check: skip if today's entry already exists for this habit.
        // This prevents duplicates when the program is run more than once per day.
        match habit_exists_today(&notion, &habit_id, &todays_id) {
            Ok(true) => {
                info!(habit = %habit_name, "Skipping — entry already exists for today");
            }
            Ok(false) => {
                if let Err(e) = daily_habits::create_daily_habit(&notion, &habit_id, &todays_id, &habit_name) {
                    eprintln!("Failed to create habit '{}': {}", habit_name, e);
                }
            }
            Err(e) => {
                eprintln!("Failed to check existence for '{}': {}", habit_name, e);
            }
        }
    }

    Ok(())
}
