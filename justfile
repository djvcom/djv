default:
    @just --list

fmt:
    cargo fmt
    nixfmt *.nix

lint:
    SQLX_OFFLINE=true cargo clippy --all-features -- -D warnings
    statix check .
    deadnix .

test:
    SQLX_OFFLINE=true DATABASE_URL="${DATABASE_URL:-postgres:///djv_test?host=/run/postgresql}" cargo test --features ssr

test-watch:
    SQLX_OFFLINE=true DATABASE_URL="${DATABASE_URL:-postgres:///djv_test?host=/run/postgresql}" cargo watch -x 'test --features ssr'

check: fmt lint test
    @echo "All checks passed"

dev:
    cargo leptos watch

build:
    cargo leptos build --release

# Database commands
db-setup:
    @echo "Creating databases..."
    createdb djv 2>/dev/null || echo "djv database already exists"
    createdb djv_dev 2>/dev/null || echo "djv_dev database already exists"
    createdb djv_test 2>/dev/null || echo "djv_test database already exists"
    @echo "Done! Databases ready."

db-migrate:
    @echo "Running migrations on $DATABASE_URL..."
    cargo run --features ssr --quiet 2>&1 | head -5 || true

db-reset:
    @echo "Resetting database..."
    dropdb --if-exists $(basename $DATABASE_URL)
    createdb $(basename $DATABASE_URL)
    @echo "Database reset. Run 'just db-migrate' to apply migrations."

db-prepare:
    @echo "Generating sqlx query cache..."
    cargo sqlx prepare -- --all-targets --all-features
    @echo "Done! Commit .sqlx/ directory to version control."

db-prepare-check:
    @echo "Verifying sqlx query cache is up-to-date..."
    cargo sqlx prepare --check -- --all-targets --all-features
