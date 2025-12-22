default:
    @just --list

fmt:
    cargo fmt

lint:
    cargo clippy --all-features -- -D warnings

test:
    cargo test

check: fmt lint test
    @echo "All checks passed"

dev:
    cargo leptos watch

build:
    cargo leptos build --release
