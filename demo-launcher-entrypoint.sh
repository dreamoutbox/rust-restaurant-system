#!/usr/bin/env bash
#
# demo-launcher-entrypoint.sh
# Sets up PostgreSQL 18, downloads the release binary, configures systemd,
# executes SQLx migrations, and seeds mock data for bare-metal demo environments.
# Must be executed as root on a Debian/Ubuntu system.

set -euo pipefail

# Configuration defaults
APP_NAME="rust-restaurant-system"
APP_DIR="/app/${APP_NAME}"
APP_USER="restaurant"
APP_GROUP="restaurant"
APP_PORT="${APP_PORT:-3000}"

DB_NAME="${DB_NAME:-restaurant}"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-example}"
DB_PORT="${DB_PORT:-5432}"
DB_HOST="${DB_HOST:-localhost}"
DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

RELEASE_TAR_URL="${RELEASE_TAR_URL:-https://github.com/dreamoutbox/rust-restaurant-system/releases/latest/download/rust-restaurant-system-linux-x86_64.tar.gz}"
SQLX_CLI_TAR_URL="${SQLX_CLI_TAR_URL:-https://github.com/cargo-bins/cargo-quickinstall/releases/download/sqlx-cli-0.8.2/sqlx-cli-0.8.2-x86_64-unknown-linux-gnu.tar.gz}"

# Verifies that current script execution has root privileges.
# Arguments:
#   None
# Example:
#   ensure_root
ensure_root() {
    if [ "$(id -u)" -ne 0 ]; then
        echo "[ERROR] This script must be run as root." >&2
        exit 1
    fi
}

# Detects the Debian or Ubuntu release codename from /etc/os-release or lsb_release.
# Arguments:
#   None
# Output:
#   Prints detected codename to stdout (e.g. "bookworm", "noble")
# Example:
#   codename="$(detect_distro_codename)"
detect_distro_codename() {
    local codename=""
    if [ -f /etc/os-release ]; then
        # shellcheck source=/dev/null
        . /etc/os-release
        codename="${VERSION_CODENAME:-${UBUNTU_CODENAME:-}}"
    fi

    if [ -z "$codename" ] && command -v lsb_release >/dev/null 2>&1; then
        codename="$(lsb_release -cs 2>/dev/null || true)"
    fi

    if [ -z "$codename" ]; then
        echo "[ERROR] Unable to detect distribution codename." >&2
        exit 1
    fi

    echo "$codename"
}

# Installs PostgreSQL 18 and client tools using the official PGDG repository.
# Arguments:
#   $1 - Distribution codename (e.g., "bookworm", "noble")
# Example:
#   install_postgresql_18 "bookworm"
install_postgresql_18() {
    local codename="$1"
    echo "[INFO] Configuring PostgreSQL APT repository for codename: ${codename}..."

    apt-get update -q
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates curl gnupg lsb-release

    install -d -m 0755 /etc/apt/keyrings
    curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc \
        | gpg --dearmor --yes -o /etc/apt/keyrings/postgresql.gpg

    echo "deb [signed-by=/etc/apt/keyrings/postgresql.gpg] http://apt.postgresql.org/pub/repos/apt ${codename}-pgdg main" \
        > /etc/apt/sources.list.d/pgdg.list

    apt-get update -q
    echo "[INFO] Installing postgresql-18 and postgresql-client-18..."
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        postgresql-18 postgresql-client-18

    systemctl enable --now postgresql
}

# Configures PostgreSQL credentials, verifies connection, and ensures target database exists.
# Arguments:
#   $1 - Database host (e.g., "localhost")
#   $2 - Database port (e.g., "5432")
#   $3 - Database user (e.g., "postgres")
#   $4 - Database password (e.g., "example")
#   $5 - Database name (e.g., "restaurant")
# Example:
#   setup_postgresql_db "localhost" "5432" "postgres" "example" "restaurant"
setup_postgresql_db() {
    local host="$1"
    local port="$2"
    local user="$3"
    local password="$4"
    local dbname="$5"

    echo "[INFO] Waiting for PostgreSQL service on ${host}:${port}..."
    local retries=30
    until pg_isready -h "$host" -p "$port" >/dev/null 2>&1 || [ "$retries" -le 0 ]; do
        sleep 1
        retries=$((retries - 1))
    done

    if [ "$retries" -le 0 ]; then
        echo "[ERROR] PostgreSQL server failed to become ready within timeout." >&2
        exit 1
    fi

    echo "[INFO] Setting password for user '${user}'..."
    su - postgres -c "psql -c \"ALTER USER ${user} WITH PASSWORD '${password}';\""

    echo "[INFO] Ensuring database '${dbname}' exists..."
    if ! su - postgres -c "psql -tc \"SELECT 1 FROM pg_database WHERE datname = '${dbname}';\"" | grep -q 1; then
        su - postgres -c "psql -c \"CREATE DATABASE ${dbname} OWNER ${user};\""
    fi

    echo "[INFO] Testing database connection..."
    PGPASSWORD="$password" psql -h "$host" -p "$port" -U "$user" -d "$dbname" -c "SELECT 1;" >/dev/null
}

# Downloads and extracts release tarball to target application directory.
# Arguments:
#   $1 - Target directory (e.g., "/app/rust-restaurant-system")
#   $2 - URL to tar.gz release asset
# Example:
#   fetch_and_extract_release "/app/rust-restaurant-system" "https://github.com/.../release.tar.gz"
fetch_and_extract_release() {
    local target_dir="$1"
    local release_url="$2"
    local tmp_archive="/tmp/rust-restaurant-system-release.tar.gz"

    echo "[INFO] Fetching release archive from: ${release_url}..."
    curl -fsSL -o "$tmp_archive" "$release_url"

    echo "[INFO] Extracting release files into: ${target_dir}..."
    mkdir -p "$target_dir"
    tar -xzf "$tmp_archive" -C "$target_dir"
    rm -f "$tmp_archive"

    chmod +x "${target_dir}/rust-restaurant-system" "${target_dir}/seed"
}

# Creates application service user and adjusts ownership and directories.
# Arguments:
#   $1 - System user name (e.g., "restaurant")
#   $2 - System group name (e.g., "restaurant")
#   $3 - Target directory (e.g., "/app/rust-restaurant-system")
#   $4 - Database URL for .env file
#   $5 - Application listen port
# Example:
#   setup_run_user_and_env "restaurant" "restaurant" "/app/rust-restaurant-system" "postgres://..." "3000"
setup_run_user_and_env() {
    local run_user="$1"
    local run_group="$2"
    local target_dir="$3"
    local db_url="$4"
    local port="$5"

    echo "[INFO] Setting up system user '${run_user}'..."
    if ! id "$run_user" >/dev/null 2>&1; then
        useradd --system --shell /usr/sbin/nologin --home-dir "$target_dir" "$run_user"
    fi

    mkdir -p "${target_dir}/uploads"

    local env_file="${target_dir}/.env"
    if [ ! -f "$env_file" ]; then
        echo "[INFO] Generating default .env file in ${target_dir}..."
        cat <<EOF > "$env_file"
DATABASE_URL=${db_url}
JWT_SECRET=super-secret-key-for-restaurant-system-jwt-token-auth-2026
JWT_EXPIRY_HOURS=24
BASE_URL=http://localhost:${port}
UPLOAD_DIR=./uploads
STRIPE_SECRET_KEY=sk_test_mock_key
STRIPE_WEBHOOK_SECRET=whsec_mock_key
RUST_LOG=info
PORT=${port}
EOF
        chmod 0640 "$env_file"
    fi

    chown -R "${run_user}:${run_group}" "$target_dir"
}

# Configures and registers systemd unit with auto-restart policy.
# Arguments:
#   $1 - Service name (e.g., "rust-restaurant-system")
#   $2 - System user (e.g., "restaurant")
#   $3 - System group (e.g., "restaurant")
#   $4 - Target directory (e.g., "/app/rust-restaurant-system")
# Example:
#   setup_systemd_service "rust-restaurant-system" "restaurant" "restaurant" "/app/rust-restaurant-system"
setup_systemd_service() {
    local service_name="$1"
    local run_user="$2"
    local run_group="$3"
    local target_dir="$4"
    local unit_file="/etc/systemd/system/${service_name}.service"

    echo "[INFO] Creating systemd service file at ${unit_file}..."
    cat <<EOF > "$unit_file"
[Unit]
Description=Rust Restaurant System Service
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=${run_user}
Group=${run_group}
WorkingDirectory=${target_dir}
ExecStart=${target_dir}/rust-restaurant-system
Restart=always
RestartSec=5s
EnvironmentFile=-${target_dir}/.env
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable "${service_name}.service"
}

# Ensures sqlx CLI is installed and applies all SQLx migrations.
# Arguments:
#   $1 - Target application directory containing migrations folder
#   $2 - Database connection URL
#   $3 - System run user
#   $4 - URL to sqlx-cli prebuilt tarball
# Example:
#   run_sqlx_migrations "/app/rust-restaurant-system" "postgres://..." "restaurant" "https://..."
run_sqlx_migrations() {
    local target_dir="$1"
    local db_url="$2"
    local run_user="$3"
    local sqlx_url="$4"

    if ! command -v sqlx >/dev/null 2>&1; then
        echo "[INFO] Installing sqlx-cli binary..."
        curl -fsSL "$sqlx_url" | tar -xz -C /usr/local/bin sqlx 2>/dev/null || true
        chmod +x /usr/local/bin/sqlx 2>/dev/null || true
    fi

    if command -v sqlx >/dev/null 2>&1; then
        echo "[INFO] Executing sqlx migrate run..."
        su -s /bin/bash "$run_user" -c "cd '${target_dir}' && sqlx migrate run --source migrations --database-url '${db_url}'"
    else
        echo "[WARN] sqlx CLI unavailable; starting service temporarily to execute embedded migrations..."
        systemctl start rust-restaurant-system.service
        sleep 3
    fi
}

# Executes the seed binary to populate mock users, tables, categories, dishes, and images.
# Arguments:
#   $1 - Target application directory containing seed binary
#   $2 - System run user
# Example:
#   run_seed_mock_data "/app/rust-restaurant-system" "restaurant"
run_seed_mock_data() {
    local target_dir="$1"
    local run_user="$2"

    echo "[INFO] Running seed to populate mock data..."
    su -s /bin/bash "$run_user" -c "cd '${target_dir}' && ./seed"
}

# Main execution flow
main() {
    ensure_root

    local codename
    codename="$(detect_distro_codename)"

    echo "[INFO] Starting demo setup for ${APP_NAME}..."
    install_postgresql_18 "$codename"
    setup_postgresql_db "$DB_HOST" "$DB_PORT" "$DB_USER" "$DB_PASSWORD" "$DB_NAME"

    fetch_and_extract_release "$APP_DIR" "$RELEASE_TAR_URL"
    setup_run_user_and_env "$APP_USER" "$APP_GROUP" "$APP_DIR" "$DATABASE_URL" "$APP_PORT"
    setup_systemd_service "$APP_NAME" "$APP_USER" "$APP_GROUP" "$APP_DIR"

    run_sqlx_migrations "$APP_DIR" "$DATABASE_URL" "$APP_USER" "$SQLX_CLI_TAR_URL"
    run_seed_mock_data "$APP_DIR" "$APP_USER"

    if command -v ufw >/dev/null 2>&1 && ufw status | grep -q "Status: active"; then
        echo "[INFO] Allowing port ${APP_PORT}/tcp through UFW..."
        ufw allow "${APP_PORT}/tcp" || true
    fi

    echo "[INFO] Starting ${APP_NAME} service..."
    systemctl restart "${APP_NAME}.service"
    systemctl is-active --quiet "${APP_NAME}.service"

    echo "[SUCCESS] Demo environment setup complete."
    echo "Application endpoint: http://<vps-ip>:${APP_PORT}"
    echo "Demo Credentials:"
    echo "  - Admin:   admin / admin"
    echo "  - Cashier: cashier1 / password"
    echo "  - Kitchen: kitchen1 / password"
    echo "  - Waiter:  waiter1 / password"
}

main "$@"
