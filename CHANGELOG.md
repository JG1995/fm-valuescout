# Changelog

All notable changes to FM ValueScout are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.3] - 2026-08-18

### Fixed

- Planner teams can now be configured per save, with editable display names and safe removal of unneeded teams.
- Planner optimization now accepts familiarity of 12 in both tactic phases and applies hidden ranking deductions below 16.

## [0.6.2] - 2026-08-18

### Fixed

- Planner optimizer eligibility now requires familiarity of 16 for the IP position and accepts 12 for a distinct OOP position.

## [0.6.1] - 2026-08-18

### Fixed

- Manager job-fit scores now use Motivating, People Management, Judging Player Ability, Judging Player Potential, and Tactical Knowledge. Staff Shortlists now display and rank a Manager preferred job by that score.

## [0.6.0] - 2026-08-17

### Added

- Staff Shortlist imports a save-owned Football Manager staff CSV, keeps Preferred Job, Club Job, and Coaching Qualifications, and shows only current matched staff.
- Staff Shortlist filters exact Preferred Job values and unemployed staff, and adapts role-score columns for direct jobs and Coach.

## [0.5.2] - 2026-08-17

### Fixed

- Tactic positions can use distinct right, centre, and left central placements without losing role compatibility or optimizer familiarity.

## [0.5.1] - 2026-08-17

### Fixed

- Scrollable suggestion menus remain visible and interactive when opened inside clipped panels, modal scroll regions, or the app shell.

## [0.5.0] - 2026-08-16

### Added

- A Staff workspace now provides full-snapshot Staff Search with configurable columns, filters, sorting, and 20 current job-fit scores.
- My Staff shows the complete configured Senior, Reserves, and Youth club family and can apply the guarded fixed +10 CA action across eligible staff.
- Staff profiles now show current Coaching, Mental, and Knowledge attributes, ranked job fit, the shared hidden-information preference, and an individual fixed +10 CA action.

### Changed

- Staff and player role scores use the shared accessible four-tier score treatment, and staff attributes match the Player Profile presentation.
- Load Data schema v8 persists the staff attributes and job-fit scores required by the Staff workspace while older snapshots remain readable.

### Fixed

- Working With Youngsters now preserves its raw 1–20 FM value instead of applying reputation scaling.
- Staff data refreshes after save or snapshot context changes, and partial My Staff boost failures retain truthful progress and applied-change feedback.

## [0.4.0] - 2026-08-16

### Added

- Player profiles now display complete FM position familiarity, including sweeper familiarity, from exact scanned values.

### Changed

- Search, Squad, Academy, Planner, optimizer, and role projection now apply explicit recorded, playable, and natural-position thresholds so position labels, best-position selection, and eligibility remain consistent with complete familiarity data.
- New **Load Data** scans use the complete schema-v7 position map while existing schema-v6 snapshots remain readable.

## [0.3.0] - 2026-08-16

### Added

- Player profiles can reveal or conceal potential and hidden information for the active save. The saved preference defaults to revealed and applies to every player in that save.
- Player profiles now use Outfield, Goalkeeping, Hidden, and Personality attribute tabs with goalkeeper-specific defaults and grouping.
- Player summaries now show the best Current IP, Current OOP, Potential IP, and Potential OOP roles.
- The desktop app and Windows bundle now use the FM ValueScout icon assets.

### Changed

- **Load Data** outcome banners can be dismissed and now clear when their originating save context is no longer active.

### Fixed

- Player profile controls and panels remain layout-stable when information is concealed or a **Load Data** outcome reduces the available workspace height.

## [0.2.0] - 2026-08-15

### Added

- **Boost all CA** and **Make all Wonderkids** now show determinate `processed / total` progress in the confirmation modal.
- Squad overview feedback now keeps the final boost result stable with updated, skipped, and failed counts, and explains when **Load Data** is required before another action.

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
