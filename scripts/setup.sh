#!/usr/bin/env bash
# setup.sh — one-time configuration for SBDailyHabits
#
# Supports two storage strategies for the Notion API token:
#   Option 1 — restricted .env file at ~/.config/sbdailyhabits/.env
#   Option 2 — OS keychain via secret-tool (recommended; token never on disk)
#
# Usage:
#   ./scripts/setup.sh

set -euo pipefail

CONFIG_DIR="$HOME/.config/sbdailyhabits"
ENV_FILE="$CONFIG_DIR/.env"

echo "========================================"
echo "  SBDailyHabits — First-time Setup"
echo "========================================"
echo

# ── Collect all config values ────────────────────────────────────────────────

read -rp "APP_NAME [SBDailyHabits]: " APP_NAME
APP_NAME="${APP_NAME:-SBDailyHabits}"

read -rp "APP_VERSION [1.0.0]: " APP_VERSION
APP_VERSION="${APP_VERSION:-1.0.0}"

read -rp "DATABASE_USER: " DATABASE_USER
read -rp "DATABASE_PASSWORD: " DATABASE_PASSWORD
read -rp "DATABASE_HOST: " DATABASE_HOST
read -rp "DATABASE_PORT [5432]: " DATABASE_PORT
DATABASE_PORT="${DATABASE_PORT:-5432}"

read -rp "NOTION_URL [https://api.notion.com/v1]: " NOTION_URL
NOTION_URL="${NOTION_URL:-https://api.notion.com/v1}"

read -rp "NOTION_VERSION [2022-06-28]: " NOTION_VERSION
NOTION_VERSION="${NOTION_VERSION:-2022-06-28}"

read -rsp "NOTION_TOKEN (input hidden): " NOTION_TOKEN
echo

read -rp "DAILY_DATABASE_ID: " DAILY_DATABASE_ID
read -rp "HABITS_DATABASE_ID: " HABITS_DATABASE_ID
read -rp "HABITS_MASTER_DATABASE_ID: " HABITS_MASTER_DATABASE_ID
read -rp "DAILY_STATS_PAGE_ID: " DAILY_STATS_PAGE_ID

echo
echo "========================================"
echo "  Token Storage"
echo "========================================"
echo "  1) Restricted .env file (simpler, token stored on disk)"
echo "  2) OS keychain via secret-tool (recommended, token never on disk)"
echo
read -rp "Choose [1/2, default 2]: " STORAGE_CHOICE
STORAGE_CHOICE="${STORAGE_CHOICE:-2}"

# ── Create config directory ───────────────────────────────────────────────────

mkdir -p "$CONFIG_DIR"

# ── Option 2: keychain ────────────────────────────────────────────────────────

if [[ "$STORAGE_CHOICE" == "2" ]]; then
    if ! command -v secret-tool &>/dev/null; then
        echo
        echo "secret-tool not found. Install it with:"
        echo "  sudo apt-get install -y libsecret-tools"
        echo
        echo "Then re-run this setup, or choose option 1."
        exit 1
    fi

    echo
    echo "Storing NOTION_TOKEN in OS keychain..."
    echo -n "$NOTION_TOKEN" | secret-tool store \
        --label="Notion Token (SBDailyHabits)" \
        service sbdailyhabits \
        key notion_token
    echo "Token stored in keychain."

    # Write .env without the token
    cat > "$ENV_FILE" <<EOF
APP_NAME=$APP_NAME
APP_VERSION=$APP_VERSION

DATABASE_USER=$DATABASE_USER
DATABASE_PASSWORD=$DATABASE_PASSWORD
DATABASE_HOST=$DATABASE_HOST
DATABASE_PORT=$DATABASE_PORT

NOTION_URL=$NOTION_URL
NOTION_VERSION=$NOTION_VERSION

# NOTION_TOKEN is loaded from the OS keychain at runtime — not stored here.

DAILY_DATABASE_ID=$DAILY_DATABASE_ID
HABITS_DATABASE_ID=$HABITS_DATABASE_ID
HABITS_MASTER_DATABASE_ID=$HABITS_MASTER_DATABASE_ID
DAILY_STATS_PAGE_ID=$DAILY_STATS_PAGE_ID
EOF

# ── Option 1: restricted .env ─────────────────────────────────────────────────

else
    cat > "$ENV_FILE" <<EOF
APP_NAME=$APP_NAME
APP_VERSION=$APP_VERSION

DATABASE_USER=$DATABASE_USER
DATABASE_PASSWORD=$DATABASE_PASSWORD
DATABASE_HOST=$DATABASE_HOST
DATABASE_PORT=$DATABASE_PORT

NOTION_URL=$NOTION_URL
NOTION_VERSION=$NOTION_VERSION
NOTION_TOKEN=$NOTION_TOKEN

DAILY_DATABASE_ID=$DAILY_DATABASE_ID
HABITS_DATABASE_ID=$HABITS_DATABASE_ID
HABITS_MASTER_DATABASE_ID=$HABITS_MASTER_DATABASE_ID
DAILY_STATS_PAGE_ID=$DAILY_STATS_PAGE_ID
EOF
fi

# Restrict permissions regardless of option — the file may contain other secrets.
chmod 600 "$ENV_FILE"
echo "Config written to $ENV_FILE (permissions: 600)"

echo
echo "========================================"
echo "  Setup complete."
echo "  Run the app with:  ./scripts/run.sh"
echo "========================================"
