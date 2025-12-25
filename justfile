default:
    @just --list

fmt:
    cargo fmt
    nixfmt *.nix

lint:
    cargo clippy --all-features -- -D warnings
    statix check .
    deadnix .

test:
    cargo test

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
    @echo "Done! Databases ready."

db-migrate:
    @echo "Running migrations on $DATABASE_URL..."
    cargo run --features ssr --quiet 2>&1 | head -5 || true

db-reset:
    @echo "Resetting database..."
    dropdb --if-exists $(basename $DATABASE_URL)
    createdb $(basename $DATABASE_URL)
    @echo "Database reset. Run 'just db-migrate' to apply migrations."
