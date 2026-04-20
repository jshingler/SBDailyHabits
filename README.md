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
scripts/
├── setup.sh           # One-time interactive configuration
└── run.sh             # Daily run wrapper (loads config + keychain token)
```

## Setup

### 1. Build the binary

```bash
cargo build --release
```

The binary is written to `target/release/sb-daily-habits`.

### 2. Run the setup script

`scripts/setup.sh` is an interactive one-time configurator. It prompts for all required values and lets you choose how to store the Notion API token:

```bash
./scripts/setup.sh
```

**Option 1 — Restricted `.env` file** *(simpler)*
Writes all config including the token to `~/.config/sbdailyhabits/.env` with `chmod 600`. The token is on disk but only readable by your user.

**Option 2 — OS keychain** *(recommended)*
Stores the Notion token in the system keychain (GNOME Keyring on Linux) via `secret-tool` — the token never touches disk as plaintext. The `.env` file is written without the token; `run.sh` injects it from the keychain at runtime.

To use Option 2, `secret-tool` must be installed:
```bash
sudo apt-get install -y libsecret-tools
```

Config is written to `~/.config/sbdailyhabits/.env`. This file is outside the project directory and will never be committed.

### Configuration values

| Variable | Description |
|---|---|
| `APP_NAME` | Application name (e.g. `SBDailyHabits`) |
| `APP_VERSION` | Application version (e.g. `1.0.0`) |
| `DATABASE_*` | Postgres connection details (reserved for future use) |
| `NOTION_TOKEN` | API token from [notion.so/my-integrations](https://www.notion.so/my-integrations) |
| `NOTION_URL` | Notion API base URL (`https://api.notion.com/v1`) |
| `NOTION_VERSION` | Notion API version (`2022-06-28`) |
| `DAILY_DATABASE_ID` | UUID of your Daily Tracking database |
| `HABITS_DATABASE_ID` | UUID of your Daily Habits database |
| `HABITS_MASTER_DATABASE_ID` | UUID of your Habits Master database |
| `DAILY_STATS_PAGE_ID` | UUID of your Daily Stats page |

Database and page IDs are the UUIDs found in each Notion page's URL.

## Running

### Manually

```bash
./scripts/run.sh
```

`run.sh` loads `~/.config/sbdailyhabits/.env`, injects `NOTION_TOKEN` from the OS keychain if available, then executes the binary. It falls back gracefully to the token in `.env` if the keychain is unavailable.

Control log verbosity with `RUST_LOG`:

```bash
RUST_LOG=debug ./scripts/run.sh   # show all log output
RUST_LOG=info ./scripts/run.sh    # show info and above (default)
```

### Automated daily run via cron

To run automatically every morning at 7am, add a cron entry:

```bash
crontab -e
```

Add this line (adjust the path to match where you cloned the repo):

```
0 7 * * * /path/to/SBDailyHabits/scripts/run.sh >> ~/.local/share/sbdailyhabits/run.log 2>&1
```

This logs all output (stdout and stderr) to `~/.local/share/sbdailyhabits/run.log`. Create the log directory first:

```bash
mkdir -p ~/.local/share/sbdailyhabits
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
