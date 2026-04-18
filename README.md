# SBDailyHabits

A Rust CLI tool that automates daily habit tracking in Notion. Run it once each morning and it automatically creates that day's habit entries across your Notion databases.

## What It Does

Each run performs three steps:

1. **Finds today's Day entry** — queries your Notion Daily Tracking database for the page matching today's date
2. **Gets your active habits** — queries the Habits Master database, filtered to `Status = Active`
3. **Creates daily habit entries** — for each active habit, creates a new entry in the Daily Habits database linked to the habit, today's Day page, and your Daily Stats page

## Notion Data Model

The tool connects four Notion databases:

| Database | Purpose |
|---|---|
| **Habits Master** | Source of truth — all habits, filtered by Active status |
| **Daily Tracking** | One entry per day (the "Day" page) |
| **Daily Habits** | One row per habit per day, linked to both above |
| **Daily Stats** | Rollup stats page linked to each daily habit entry |

## Project Structure

```
src/
├── main.rs            # Entry point — orchestrates the daily run
├── config.rs          # Loads config.properties via once_cell singleton
├── daily_tracking.rs  # Queries Notion for today's Day entry ID
├── habits_md.rs       # Queries Notion for active habits list
└── daily_habits.rs    # Creates daily habit entries in Notion
config/
├── config.properties  # API keys and Notion database IDs (not committed)
└── log4rs.yaml        # Logging configuration
```

## Configuration

Create `config/config.properties` with the following keys:

```properties
app.name=SBDailyHabits
app.version=1.0.0

database.user=
database.password=
database.host=
database.port=

notion.api.url=https://api.notion.com/v1
notion.api.version=2022-06-28
notion.api.token=secret_YOUR_TOKEN_HERE

daily.database.id=YOUR_DAILY_TRACKING_DB_ID
habits.database.id=YOUR_DAILY_HABITS_DB_ID
habits.master.database.id=YOUR_HABITS_MASTER_DB_ID
daily.stats.page.id=YOUR_DAILY_STATS_PAGE_ID
```

Your Notion API token can be created at [notion.so/my-integrations](https://www.notion.so/my-integrations). The database IDs are the UUIDs found in each database's Notion URL.

## Running

```bash
cargo run
```

## Dependencies

- [`reqwest`](https://crates.io/crates/reqwest) — blocking HTTP client for Notion API calls
- [`serde`](https://crates.io/crates/serde) / [`serde_json`](https://crates.io/crates/serde_json) — JSON serialization
- [`serde-java-properties`](https://crates.io/crates/serde-java-properties) — `.properties` file config parsing
- [`chrono`](https://crates.io/crates/chrono) — date formatting for today's query
- [`log`](https://crates.io/crates/log) / [`log4rs`](https://crates.io/crates/log4rs) — structured logging
- [`once_cell`](https://crates.io/crates/once_cell) — lazy config singleton
