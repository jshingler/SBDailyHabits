use sb_daily_habits::config;
use sb_daily_habits::error::HabitsError;
use sb_daily_habits::notion_client::NotionClient;
use sb_daily_habits::daily_tracking;
use sb_daily_habits::habits_md;
use sb_daily_habits::daily_habits::{self, habit_exists_today};
use log::info;

type Result<T> = std::result::Result<T, HabitsError>;

fn main() -> Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default())
        .map_err(|e| HabitsError::Config(e.to_string()))?;

    info!("App Name: {}", &config::CONFIG.app.name);
    info!("App Version: {}", &config::CONFIG.app.version);

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
                info!("Skipping '{}' — entry already exists for today", habit_name);
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
