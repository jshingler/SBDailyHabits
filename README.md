# SBDailyHabits

A Rust CLI tool that automates daily habit tracking in Notion. Run it once each morning and it automatically creates that day's habit entries across your Notion databases.

## What It Does

Each run performs three steps:

1. **Finds today's Day entry** — queries your Notion Daily Tracking database for the page matching today's date
2. **Gets your active habits** — queries the Habits Master database, filtered to `Status = Active`
3. **Creates daily habit entries** — for each active habit, creates a new entry in the Daily Habits database linked to the habit, today's Day page, and your Daily Stats page

Runs are idempotent — if you run it twice in a day, it skips habits that already have an entry for today.

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
├── config.rs          # Loads .env config via dotenvy/envy singleton
├── error.rs           # Custom HabitsError type (thiserror)
├── notion_client.rs   # NotionClient struct for dependency injection
├── daily_tracking.rs  # Queries Notion for today's Day entry ID
├── habits_md.rs       # Queries Notion for active habits list
└── daily_habits.rs    # Creates daily habit entries in Notion
tests/
└── notion_integration.rs  # Wiremock integration tests
```

## Configuration

Copy `.env.example` to `.env` and fill in your values:

```bash
cp .env.example .env
```

```env
APP_NAME=SBDailyHabits
APP_VERSION=1.0.0

DATABASE_USER=
DATABASE_PASSWORD=
DATABASE_HOST=
DATABASE_PORT=5432

NOTION_TOKEN=secret_YOUR_TOKEN_HERE
NOTION_URL=https://api.notion.com/v1
NOTION_VERSION=2022-06-28

DAILY_DATABASE_ID=your-daily-tracking-db-id
HABITS_DATABASE_ID=your-daily-habits-db-id
HABITS_MASTER_DATABASE_ID=your-habits-master-db-id
DAILY_STATS_PAGE_ID=your-daily-stats-page-id
```

Your Notion API token can be created at [notion.so/my-integrations](https://www.notion.so/my-integrations). The database IDs are the UUIDs found in each database's Notion URL.

The `.env` file is listed in `.gitignore` and will never be committed.

## Running

```bash
cargo run
```

Control log verbosity with `RUST_LOG`:

```bash
RUST_LOG=debug cargo run   # show all log output
RUST_LOG=info cargo run    # show info and above (default)
```

## Testing

```bash
cargo test
```

## CI Pipeline

Every push and pull request to `main` automatically runs:

1. **Tests** — `cargo test` (all tests must pass)
2. **Coverage** — generates an HTML coverage report
3. **Lint** — `cargo clippy -- -D warnings` (zero warnings allowed)

### Viewing the Coverage Report

1. Go to the repo on GitHub → **Actions** tab
2. Click the CI run
3. Scroll to the bottom → **Artifacts** section
4. Download **coverage-report** (zip)
5. Unzip and open `index.html` in your browser

Green lines were executed during tests; red lines were not.

### Publishing a Release

The release job only runs when you push a version tag — it does **not** run on regular branch pushes. To publish a release:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This will:
1. Run all tests first (release is blocked if tests fail)
2. Build an optimized binary (`cargo build --release`)
3. Create a GitHub Release with the binary attached as `sb-daily-habits-linux-x86_64`
4. Auto-generate release notes from commit messages

## Dependencies

- [`reqwest`](https://crates.io/crates/reqwest) — blocking HTTP client for Notion API calls
- [`serde`](https://crates.io/crates/serde) / [`serde_json`](https://crates.io/crates/serde_json) — JSON serialization
- [`dotenvy`](https://crates.io/crates/dotenvy) / [`envy`](https://crates.io/crates/envy) — `.env` file and environment variable config
- [`chrono`](https://crates.io/crates/chrono) — date formatting for today's query
- [`tracing`](https://crates.io/crates/tracing) / [`tracing-subscriber`](https://crates.io/crates/tracing-subscriber) — structured key=value logging
- [`thiserror`](https://crates.io/crates/thiserror) — custom error types
- [`once_cell`](https://crates.io/crates/once_cell) — lazy config singleton
