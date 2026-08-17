# Staff Shortlist CSV Enrichment

## Intent

Let a user import a filtered Football Manager staff CSV and inspect realistic recruitment options without treating CSV values as current staff facts.

## Delivered behavior

- `/staff` has a third **Shortlist** tab. It prompts for an upload when the active save has no shortlist.
- The native picker imports a UTF-8, semicolon-delimited CSV with `Unique ID`, `Preferred Job`, `Club Job`, and `Coaching Qualifications`. A replacement upload warns before file selection.
- A successful import replaces only the active save's prior shortlist with exact UID matches in the effective current snapshot. The closed modal leaves a visible stored, total, and skipped summary. Invalid, stale, zero-match, or failed imports leave the prior shortlist unchanged.
- The table filters by an exact Preferred Job and by an unemployment rule that treats trimmed blank and `-` Club Job values as unemployed.
- All jobs keeps an independent configurable layout. A mapped job shows one matching job-fit score and ranks it descending. Coach shows six outfield coaching scores without choosing a score sort. Manager and unrecognized jobs show no extra score and rank by CA.
- Shortlist rows continue to open the existing Staff Profile.

## Final architecture

- Migration v27 adds `staff_shortlist_entries(save_id, staff_uid, preferred_job, club_job, coaching_qualifications)`. The `(save_id, staff_uid)` key allows one active shortlist per save. Rows cascade when their save is deleted but survive snapshot replacement.
- Rust `features/csv_import::staff_shortlist` validates the bounded file, captures active save and current-snapshot context, parses outside the database lock, revalidates context in the transaction, and atomically replaces matching rows.
- Rust `list_staff_shortlist` joins save-owned entries to the effective current snapshot by UID. It applies Preferred Job and unemployment filters before count, sort, and paging, and returns shortlist metadata only for Shortlist scope.
- React owns the context-bound dialog, route state, filters, table layout, and contextual score presentation. A context change closes the dialog and suppresses late selection or import results.

## Important decisions

- CSV Preferred Job and Club Job are recruitment metadata. They do not replace extracted contract `job_id` or calculated staff suitability scores.
- The shortlist is save-owned, so it persists across app restarts and later snapshot loads. Current snapshot staff data remains authoritative.
- Imports replace rather than merge. A zero-match import does not clear the existing shortlist.
- No ADR was needed. The persistence, transactional import, and bounded query follow established repository patterns.

## Migration and operational implications

- Existing databases migrate to v27 without backfilling shortlist entries.
- The same 1 MiB trusted-file limit applies, with a staff-specific 10,000-row limit. Player CSV imports retain their 1,000-row limit and behavior.
- Assembled-app validation remains useful for the native picker, a real 2,180-row export, restart persistence, snapshot changes, and save switching.

## Validation

- Feature range: `03b0b5f` through `5b91724`, including release preparation `8b5a33d` and close-out corrections `453213b`, `8e01897`, and `5b91724`.
- `./scripts/dev test` passed: 45 files and 478 tests.
- `./scripts/dev check` passed. The only output note was the existing Biome schema/CLI version mismatch.
- `CI=1 ./scripts/dev smoke` passed.
- `./scripts/dev release-metadata v0.5.2 minor` validated version `0.6.0` and tag `v0.6.0`.
- The final feature review cleared after the close-out corrections.

## Follow-up

- The branch is ready for publication. No pull request has been published or merged.
