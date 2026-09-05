# Changelog

All notable changes to FM ValueScout are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.17.0] - 2026-09-05

### Added

- Player and Staff Search now integrate save-owned shortlist workflows, including CSV imports, shortlist filtering, and staff assignment recommendations.
- Grouped top navigation provides direct access to Home, player and staff search, My Club workspaces including Youth, and Settings.
- Snapshot history can edit an imported snapshot's in-game date.
- Squad shows a Suggested Training focus for developing players, using explicit Planner assignments first and each unassigned player's best-fit tactic lane as a fallback.

### Changed

- General Player Search shortlist filtering now remains independent of Moneyball cohort membership.
- The top utility bar now shows the app logo, a wider global search, and the active snapshot's in-game date while keeping save and data-loading actions together.

### Removed

- Removed the standalone Player Shortlist destination, the My Club Staff Shortlist workspace, the persistent left navigation rail, redundant workspace tabs, frontend player-cap controls, and the snapshot freshness indicator.

## [0.16.0] - 2026-09-03

### Added

- Player Search General, Moneyball, and Shortlist tables now provide optional Current and Potential tactic-lane columns with stable sorting and saved table layouts.
- General scoring now includes the 11 missing FM26 generic out-of-possession roles across Search, player profiles, and Planner. All 88 Moneyball roles now map to General scores.

### Changed

- Channel Midfielder now supports both attacking and central midfield positions.
- Existing score-model version 1 data remains intact but requires a normal **Load Data** run before the expanded version 2 role scores become available.

## [0.15.0] - 2026-09-02

### Added

- Player Search now provides a Shortlist tab for the current Moneyball cohort, with General attributes, filters, sorting, profile analysis, and an independent table layout.
- **Load Data** now reports Scan, Preparing, Scoring, Saving, and Finalizing progress, with phase counts and detailed timing after completion.

### Changed

- Current player and staff role metrics now use compact storage in a fresh `app-v2.db`. The app leaves the legacy `app.db` untouched; verify the new database before manually deleting the old file.

### Fixed

- CSV imports now accept Football Manager exports up to 8 MiB and 10,000 rows.

## [0.14.3] - 2026-08-30

### Fixed

- Position displays now use a consistent canonical order across player, Planner, Search, Squad, Academy, and Moneyball views.
- Academy player lists now show only playable positions with familiarity of 16 or higher, ordered by familiarity and canonical position order for ties.

## [0.14.2] - 2026-08-29

### Fixed

- Staff optimization no longer closes the app when assigning General Coaches in packaged builds.
- Planner optimization now retains automatic lane assignments in packaged builds.

## [0.14.1] - 2026-08-29

### Changed

- Staff assignment slots now follow the FM26 Coaching, Recruitment, and Medical catalog, show club-wide roles once, support Recruitment Analysts, and retain accepted optimizer results when collapsed.
- The optimizer reserves candidates for Head and Chief roles before ordinary slots and matches General Coaches to their required coaching specializations.
- Existing assignment slot counts reset once to adopt the FM26 catalog, while legacy Set Piece Coach targets consolidate into one Club slot.

### Fixed

- Coaches slots now require the correct General, Goalkeeping, and Fitness composition, preserving typed vacancies when a required coach type is unavailable.
- Configure Slots and optimizer results now share canonical squad and role ordering, with every Head role before its ordinary counterpart.

## [0.14.0] - 2026-08-26

### Added

- Squad now provides the shared Moneyball CSV import, and repeated imports accumulate data without removing omitted players.
- Staff Shortlist can define team-specific staffing targets and optimize assignments by job.

### Changed

- Player Search and Squad keep current rows visible during sorting and use indexed ordering for more responsive large-save tables.
- Current-snapshot potential attributes and role scores are materialized once and reused across Search, Squad, Planner, and player profiles.

### Fixed

- Navigation no longer runs repeated snapshot-wide potential-score validation for players and roles unrelated to the requested view.

## [0.13.0] - 2026-08-24

### Added

- My Club now provides one save-owned Club DNA definition with configurable player attributes.
- Player Search and Squad can display and sort Club DNA scores, and Player Search can filter them.

### Changed

- Back and Forward controls now sit before global search in the app header.
- Player profiles now use accessible nationality flags, compact identity and action layouts, and omit the unsupported sweeper position.
- Potential attributes and role scores now match current values for players aged 29 and older.
- Red-tier position familiarity values use muted surfaces while preserving readable and selected-state contrast.

## [0.12.0] - 2026-08-23

### Added

- The app header now provides Back and Forward controls for the current session's route history.
- Moneyball Player Profiles now show their natural-position comparison basis and clearly mark unavailable scores.

### Changed

- Managed-club search and save controls share one responsive row.
- Shared player tables now deduplicate secondary nationalities and stack player identity above club and division; saved layouts migrate duplicate identity columns.
- Moneyball Player Profile scoring now uses natural-position cohorts, and General and Moneyball headers keep stable geometry.

### Fixed

- Staff and Staff Shortlist scrolling remains contained in My Club workspaces.

## [0.11.1] - 2026-08-22

### Fixed

- Managed-club selection no longer restores an outdated club after the active save or snapshot context refreshes.

## [0.11.0] - 2026-08-21

### Added

- Planner now provides a Best role fit reference that independently ranks managed-club players for every IP and OOP tactic role, with current and potential scores.
- The Best role fit reference can switch tactic phases and score basis and sort players by role score.

## [0.10.0] - 2026-08-20

### Added

- Moneyball now provides 88 versioned, position-family-specific role scores from imported performance percentiles.
- Moneyball Player Profiles show best IP/OOP role summaries, a position-filtered role-fit panel, and score explanations.
- Moneyball Player Search adds optional role-score columns, numeric filters, and sorting for Full CSV and filtered comparison cohorts.

## [0.9.0] - 2026-08-20

### Added

- Moneyball CSV imports now store a current-snapshot player cohort with computed metrics and whole-cohort percentile scores.
- Player Search now provides an optional Moneyball view with cohort-only results, metric filters, configurable columns, and virtualized results.
- Player Profiles now provide a Moneyball analysis view, and Settings can make it the default analysis view.

## [0.8.0] - 2026-08-19

### Added

- My Club now provides URL-backed Squad, Planner, Tactic, Staff, and Staff Shortlist workspaces for the selected managed club.

### Changed

- Managed-club selection now lives in My Club, while Settings remains focused on save, snapshot, and bridge management.
- Player Search and Staff Search are explicit standalone destinations, and legacy Planner and Staff workspace links redirect to their canonical My Club URLs.

## [0.7.0] - 2026-08-18

### Added

- Settings now manages saves, snapshots, the FM bridge, and one save-scoped managed club selected from the latest snapshot.

### Changed

- Squad, Planner, Academy, My Staff, and club-wide boosts now derive membership from exact managed-club matches in the latest snapshot.
- Planner teams now share the complete managed-club player pool while preserving age limits, team priority, and save-wide uniqueness.
- Dashboard is now a placeholder while active-save selection and **Load Data** remain in the top bar.

### Removed

- Removed manual Senior, Reserves, and Youth club-source configuration, the Dashboard CSV importer, and the Dashboard sanity-player list.

## [0.6.4] - 2026-08-18

### Fixed

- Planner optimization now accepts familiarity of 12 in both tactic phases and applies hidden ranking deductions below 16.

## [0.6.3] - 2026-08-18

### Fixed

- Planner teams can now be configured per save, with editable display names and safe removal of unneeded teams.

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
