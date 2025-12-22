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
