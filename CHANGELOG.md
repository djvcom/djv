# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/djvcom/djv/compare/v0.5.0...v0.5.1) - 2025-12-26

### Other

- *(ci)* cache Playwright browsers and fix update-stable trigger ([#18](https://github.com/djvcom/djv/pull/18))

## [0.5.0](https://github.com/djvcom/djv/compare/v0.4.1...v0.5.0) - 2025-12-26

### Added

- add comprehensive test suite, UI improvements, and crates.io sync ([#16](https://github.com/djvcom/djv/pull/16))

## [0.4.1](https://github.com/djvcom/djv/compare/v0.4.0...v0.4.1) - 2025-12-24

### Other

- *(ci)* require all jobs to pass before updating stable tag

## [0.4.0](https://github.com/djvcom/djv/compare/v0.3.0...v0.4.0) - 2025-12-24

### Added

- *(otel)* improve trace quality and semantic convention compliance ([#13](https://github.com/djvcom/djv/pull/13))

## [0.3.0](https://github.com/djvcom/djv/compare/v0.2.1...v0.3.0) - 2025-12-24

### Added

- *(datadog)* add service catalog and security scanning

## [0.2.1](https://github.com/djvcom/djv/compare/v0.2.0...v0.2.1) - 2025-12-23

### Other

- *(ci)* download pre-built release-plz binary

## [0.2.0](https://github.com/djvcom/djv/compare/v0.1.1...v0.2.0) - 2025-12-23

### Added

- [**breaking**] replace unix socket with TCP binding
- *(otel)* use axum-tracing-opentelemetry for proper OTel semantics ([#5](https://github.com/djvcom/djv/pull/5))

### Fixed

- *(release)* enable minor bumps for feat commits in 0.x

### Other

- trigger release detection
- switch to TCP binding and runtime VCS attributes ([#7](https://github.com/djvcom/djv/pull/7))
- *(deps)* update opentelemetry-configuration to 0.1.2 ([#6](https://github.com/djvcom/djv/pull/6))

## [0.1.1](https://github.com/djvcom/djv/releases/tag/v0.1.1) - 2025-12-22

### Fixed

- *(nix)* add configurable group for socket permissions

### Other

- add stable tag workflow for NixOS deployments

## [0.1.0](https://github.com/djvcom/djv/releases/tag/v0.1.0) - 2025-12-22

### Added

- add OpenTelemetry instrumentation and NixOS deployment
- *(homepage)* add skeleton layout with projects

### Fixed

- *(nix)* configure crane for cargo-leptos builds
- *(ci)* use nightly toolchain and fix nix build
- *(nix)* use crane for sandboxed builds

### Other

- only run nix-build on pull requests
- add comprehensive CI workflow and update action versions
- add release-plz workflow and config
- add justfile for common tasks
- init leptos project

