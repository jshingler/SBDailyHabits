mod config;
mod habits_md;
mod daily_tracking;
mod daily_habits;

use log::info;

// `Box<dyn std::error::Error + Send + Sync>` is a common type alias for
// "any error type." Using it as main's return type means we can use `?`
// throughout main to propagate errors cleanly instead of .unwrap()-ing everything.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default())?;

    info!("App Name: {}", &config::CONFIG.app.name);
    info!("App Version: {}", &config::CONFIG.app.version);

    // `?` here means: if get_today_id() returns an Err, stop and return that
    // error from main. The program exits cleanly with an error message rather
    // than panicking with a cryptic index-out-of-bounds message.
    let todays_id = daily_tracking::get_today_id()?;

    let habits = habits_md::get_hmd()?;

    for (habit_id, habit_name) in habits {
        // `if let Err(e)` handles the error case without unwrapping.
        // Here we log failures and continue the loop rather than aborting —
        // one failed habit entry shouldn't stop the rest from being created.
        if let Err(e) = daily_habits::create_daily_habit(
            &habit_id,
            &todays_id,
            &habit_name,
            &config::CONFIG.notion.daily_stats_page_id,
        ) {
            // `eprintln!` writes to stderr — appropriate for error messages.
            eprintln!("Failed to create habit '{}': {}", habit_name, e);
        }
    }

    Ok(())
}
