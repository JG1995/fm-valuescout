# Changelog

All notable changes to FM ValueScout are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.1] - 2026-08-14

### Added

- Initial Windows super-early-alpha release with a local FM26 BepInEx bridge, snapshot-backed player search and profiles, Squad, Planner, Academy, configurable tables, CSV enrichment, and guarded player-boost actions.
- A bundled bridge build, checksum, bounded local diagnostics, and a manual recovery path for the unsigned Windows installer.
- Automatic prerelease publication only after a successful required `Check` push run on `main`, with the exact dated section as its GitHub release notes.

### Fixed

- Windows release validation now preserves the prepared changelog notes across line-ending conventions.
- Release publication now preserves absent GitHub SHAs as null instead of treating them as orphaned tags.

### Security

- Local-only application data, with no accounts, cloud sync, telemetry, or automatic data upload.
