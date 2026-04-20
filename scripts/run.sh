#!/usr/bin/env bash
# run.sh — wrapper script for SBDailyHabits
#
# Loads config from ~/.config/sbdailyhabits/.env and, if secret-tool is
# available, injects NOTION_TOKEN from the OS keychain so it never sits
# in a plaintext file. Falls back to the token in .env if keychain lookup
# fails or secret-tool is not installed.
#
# Usage:
#   ./scripts/run.sh
#
# To run daily via cron (example: 7am every day):
#   0 7 * * * /path/to/SBDailyHabits/scripts/run.sh >> ~/.local/share/sbdailyhabits/run.log 2>&1

set -euo pipefail

CONFIG_FILE="$HOME/.config/sbdailyhabits/.env"
BINARY="${SBDAILYHABITS_BIN:-$(dirname "$0")/../target/release/sb-daily-habits}"

# ── Verify config file exists ─────────────────────────────────────────────────

if [[ ! -f "$BINARY" ]]; then
    echo "ERROR: binary not found at $BINARY"
    echo "Build it with:  cargo build --release"
    exit 1
fi

if [[ ! -f "$CONFIG_FILE" ]]; then
    echo "ERROR: config not found at $CONFIG_FILE"
    echo "Run setup first:  ./scripts/setup.sh"
    exit 1
fi

# ── Load .env into environment ────────────────────────────────────────────────

# Export each non-comment, non-empty line from the .env file.
set -o allexport
# shellcheck source=/dev/null
source "$CONFIG_FILE"
set +o allexport

# ── Inject token from OS keychain (Option 2) ─────────────────────────────────

if command -v secret-tool &>/dev/null; then
    KEYCHAIN_TOKEN=$(secret-tool lookup service sbdailyhabits key notion_token 2>/dev/null || true)
    if [[ -n "$KEYCHAIN_TOKEN" ]]; then
        export NOTION_TOKEN="$KEYCHAIN_TOKEN"
    else
        echo "WARNING: secret-tool installed but no token found in keychain."
        echo "         Falling back to NOTION_TOKEN from $CONFIG_FILE"
        echo "         Re-run setup to store the token: ./scripts/setup.sh"
    fi
else
    # secret-tool not installed — rely on the token in .env (Option 1).
    : # NOTION_TOKEN already exported from source above
fi

# ── Run the binary ────────────────────────────────────────────────────────────

exec "$BINARY"
