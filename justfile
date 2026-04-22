pgdata := justfile_directory() / ".pgdata"
default_test_db_url := "postgres:///djv_test?host=" + pgdata
default_dev_db_url := "postgres:///djv_dev?host=" + pgdata

default:
    @just --list

fmt:
    cargo fmt
    nixfmt *.nix

lint:
    SQLX_OFFLINE=true cargo clippy --all-features -- -D warnings
    statix check .
    deadnix .

# Ensure the project-local cluster is running, but only if DATABASE_URL
# points at it (or is unset). Leaves CI / user-pointed-elsewhere alone.
_maybe-db-up:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${DATABASE_URL:-}" ] || [[ "${DATABASE_URL}" == *"host={{ pgdata }}"* ]]; then
        just db-up
    fi

test: _maybe-db-up
    #!/usr/bin/env bash
    set -euo pipefail
    export DATABASE_URL="${DATABASE_URL:-{{ default_test_db_url }}}"
    SQLX_OFFLINE=true cargo test --features ssr

test-watch: _maybe-db-up
    #!/usr/bin/env bash
    set -euo pipefail
    export DATABASE_URL="${DATABASE_URL:-{{ default_test_db_url }}}"
    SQLX_OFFLINE=true cargo watch -x 'test --features ssr'

check: fmt lint test
    @echo "All checks passed"

# Dev server variants
dev: _maybe-db-up
    #!/usr/bin/env bash
    set -euo pipefail
    export DATABASE_URL="${DATABASE_URL:-{{ default_dev_db_url }}}"
    cargo leptos watch

dev-no-db:
    @echo "Starting dev server without database..."
    DATABASE_URL="" cargo leptos watch

dev-empty-db: db-up
    DATABASE_URL="{{ default_test_db_url }}" cargo leptos watch

build:
    cargo leptos build --release

# Database commands
db-up:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -s "{{ pgdata }}/PG_VERSION" ]; then
        echo "Initialising cluster at {{ pgdata }}..."
        initdb --auth=trust --username="$USER" --encoding=UTF8 -D "{{ pgdata }}" >/dev/null
    fi
    if ! pg_ctl -D "{{ pgdata }}" status >/dev/null 2>&1; then
        echo "Starting postgres (socket: {{ pgdata }})..."
        pg_ctl -D "{{ pgdata }}" -l "{{ pgdata }}/postgres.log" -o "-k {{ pgdata }} -h ''" -w start >/dev/null
    fi
    for db in djv djv_dev djv_test; do
        createdb -h "{{ pgdata }}" "$db" 2>/dev/null || true
    done
    for db in djv djv_dev; do
        DATABASE_URL="postgres:///$db?host={{ pgdata }}" sqlx migrate run --source ./migrations
    done

db-down:
    -pg_ctl -D "{{ pgdata }}" stop -m fast

db-status:
    -pg_ctl -D "{{ pgdata }}" status

db-psql db="djv_dev":
    psql -h "{{ pgdata }}" -d "{{ db }}"

db-migrate: _maybe-db-up
    #!/usr/bin/env bash
    set -euo pipefail
    export DATABASE_URL="${DATABASE_URL:-{{ default_dev_db_url }}}"
    echo "Running migrations on $DATABASE_URL..."
    sqlx migrate run --source ./migrations

db-reset:
    #!/usr/bin/env bash
    set -euo pipefail
    url="${DATABASE_URL:-{{ default_dev_db_url }}}"
    db="${url##*/}"
    db="${db%%\?*}"
    echo "Resetting database $db..."
    dropdb -h "{{ pgdata }}" --if-exists "$db"
    createdb -h "{{ pgdata }}" "$db"
    echo "Database reset. Run 'just db-migrate' to apply migrations."

db-prepare:
    @echo "Generating sqlx query cache..."
    cargo sqlx prepare -- --all-targets --all-features
    @echo "Done! Commit .sqlx/ directory to version control."

db-prepare-check:
    @echo "Verifying sqlx query cache is up-to-date..."
    cargo sqlx prepare --check -- --all-targets --all-features
