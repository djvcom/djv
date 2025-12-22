# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
