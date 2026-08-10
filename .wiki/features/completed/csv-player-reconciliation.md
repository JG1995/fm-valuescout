# CSV Parsing and Player Reconciliation

## Intent

Add a safe, reusable parser for the established Youth Tracker and Moneyball Football Manager exports. Let the user select one CSV and preview how its numeric FM player UIDs match the active save's memory-backed snapshot without changing application data.

## Delivered behavior

- The Dashboard includes a secondary **CSV reconciliation** panel. With a current snapshot loaded, **Choose CSV** opens the native file picker for one `.csv` file.
- Rust detects the pinned Youth Tracker or Moneyball format from its headers, parses the complete file, and preserves blank or unavailable values as `null`. Moneyball detection runs before the broader Youth Tracker aliases.
- The preview reports the detected format, total parsed player rows, exact UID matches, unmatched rows, and bounded safe errors. Matching uses numeric UID equality only and never falls back to names.
- Unsupported headers, malformed required values, duplicate UIDs, invalid UTF-8, non-regular files, oversized files, excessive row counts, and stale save or snapshot context fail without partial output.
- The preview reads and parses outside the SQLite mutex, revalidates the captured active save and current snapshot before returning, and returns only a bounded summary. It does not expose a local path, raw CSV rows, or complete parsed data.
- Save switches and successful Load Data changes clear a completed or pending preview. A late result from the old context cannot restore the panel.
- Previewing a CSV does not write SQLite, change the snapshot, overlay CSV fields, invalidate domain caches, retain the file, or restore state after restart. Live memory-backed values remain authoritative.
- Real desktop validation confirmed native dialog cancellation, valid Youth Tracker and Moneyball selections, and same-save UID reconciliation. Browser tests and smoke use an honest dialog/IPC stub for their portion of the flow.

## Final architecture

- Rust `src-tauri/src/features/csv_import/` owns standards-compliant CSV parsing, Youth Tracker and Moneyball models, header detection, explicit normalization, UID and duplicate validation, file limits, safe errors, and preview reconciliation. `preview_csv_matches` captures the active save, current snapshot, and snapshot UIDs under a brief database lock, reads and parses a regular UTF-8 file outside that lock, then revalidates the same context.
- The parser accepts the pinned comma/semicolon Youth Tracker dialect and strict semicolon Moneyball dialect, optional UTF-8 BOM, the accepted aliases, and only values physically present in the exports. The preview boundary is 1 MiB and 1,000 player rows.
- The command DTO contains `format`, `totalPlayers`, `matchedPlayers`, and `unmatchedPlayers`. It does not carry parsed player rows, file contents, or machine-local paths. No migration or CSV table was added; the checked-in schema remains migration v16.
- React `src/features/csv-import/` owns the typed command wrapper, native dialog call, local mutation/result state, and Dashboard panel. The main capability adds only `dialog:allow-open` for the dialog plugin; the component requests one non-directory CSV selection and keeps the top-bar **Load Data** action primary.
- Checked-in fixtures record the exact Youth Tracker Monza source and the accepted Moneyball source, including source and repository SHA-256 fingerprints in `src-tauri/src/features/csv_import/fixtures/README.md`.

## Important decisions

- Use exact numeric FM UIDs for reconciliation. A player name is descriptive data, not an identity key.
- Parse in Rust at the trust boundary and return only bounded summaries, following [ADR 0014 — Rust backend and IPC trust boundary](../../decisions/0014-rust-backend-ipc-boundary.md).
- Keep file reads and parsing outside the SQLite mutex, and revalidate the captured save and snapshot before reporting matches.
- Reject duplicate UIDs and malformed populated values. Do not coerce missing or malformed statistics to zero.
- Keep parsed rows ephemeral. CSV persistence, source precedence, provenance, retention, and calculations for statistics absent from exports remain a separate follow-up.
- No new ADR or debug report was needed. The feature uses the existing Rust-owned SQLite boundary from [ADR 0015 — SQLite with Rust-owned migrations and queries](../../decisions/0015-sqlite-rust-owned.md) without changing persistence.

## Migration and operational implications

- No database migration, table, write path, cache overlay, or retained import state was added. Existing snapshots, Planner rows, Academy rows, and schema version remain unchanged after success or failure.
- The selected file stays local and is read only for the preview command. The command enforces the regular-file, extension, UTF-8, byte, and row limits before returning a result.
- A future CSV enrichment feature must design persistence and derived Moneyball statistics together, preserve memory-backed values when both sources provide a field, and decide provenance, replacement, retention, and consumers before adding a schema or read model.

## Validation

- `./scripts/dev format` made no changes.
- Frontend tests passed: 25 files and 251 tests.
- `./scripts/dev check` passed the Rust and frontend gates: 315 Rust tests passed and 2 were ignored.
- `./scripts/dev smoke` passed 20 Playwright tests. Smoke proves the browser stub path, not the native OS dialog or real file reads.
- Developer desktop validation confirmed native dialog cancel, one valid Youth Tracker preview, one valid Moneyball preview, and same-save CSV-to-bridge UID reconciliation.
- Feature-complete review used Sol High and cleared after one correction round. It retained no Critical, High, Medium, or Nitpick findings. Repowise freshness was unavailable and remains advisory only.

## Exact implementation refs

| Ref | Subject |
| --- | --- |
| `4c53660056a2c5fa61434c63b5ac7e1bcc2dee25` | `docs(import): pin CSV fixture contracts` |
| `c4e2df54bb9fdf69ca24175de56924e9b640ab26` | `feat(import): parse Youth Tracker CSV exports` |
| `8212e28893fb5b09923a5c03d44d7000727e172e` | `feat(import): parse Moneyball CSV exports` |
| `d9d53de6546e8a98b6dc188103ed4f2cf30ca3ef` | `feat(import): preview CSV matches by player UID` |
| `60d069f7a87f70e4b5f152abafad1881482a9fa` | `feat(import): add CSV reconciliation preview` |
| `6f88ae29e80a5561dc28d54125bef807fa5211ec` | `fix(import): harden CSV preview validation` |

The planning commit `81ea4f70a88d202e9411ca69c081dd32494f7f27` is intentionally excluded from this implementation set.

## Delivery profiles

| Commit | Implementation profile | Review profile | Result | Deviation |
| --- | --- | --- | --- | --- |
| `c4e2df5` — Youth Tracker parser | Terra xhigh | Sol High | Accepted after one correction pass | Exact aliases replaced a permissive substring fallback. |
| `8212e28` — Moneyball parser | Terra xhigh | Sol High | Accepted after one correction pass | Fixture CRLF was normalized to LF; source and checked-in hashes remain recorded. |
| `d9d53de` — UID preview command | Terra xhigh | Sol High | Accepted after one correction round | The row limit entered the parser to prevent pre-rejection allocation; the opened-handle read was capped after review. |
| `60d069f7` — Dashboard preview | Terra xhigh | Sol High | Accepted after one correction round | Native dialog remained a feature-level validation gap until the developer's desktop check; browser tests stub it honestly. |
| `6f88ae2` — validation hardening | Correction | Sol High feature review | Cleared in the final review | Hardened Moneyball detection and safe diagnostics. |

## Final publication

```yaml
status: published
pr_status: draft
pr_created: true
merge_status: not_merged
pr_ref: "https://github.com/JG1995/fm-valuescout/pull/40"
merge_ref: "Not merged"
branch: feature/csv-player-reconciliation
base_branch: main
provisional_pr_title: "feat(import): preview supported FM CSV exports"
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
build_feature_loop_profile: terra_xhigh
feature_close_out: current
feature_review_profile: sol_high
feature_review_blocking: false
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
project_fit: conforms
feature_review_action: skip
feature_review_recommendation: accept
ci_repair_rounds: 0
implementation_range: "exact refs listed above; planning 81ea4f70a88d202e9411ca69c081dd32494f7f27 excluded"
implementation_refs:
  - 4c53660056a2c5fa61434c63b5ac7e1bcc2dee25
  - c4e2df54bb9fdf69ca24175de56924e9b640ab26
  - 8212e28893fb5b09923a5c03d44d7000727e172e
  - d9d53de6546e8a98b6dc188103ed4f2cf30ca3ef
  - 60d069f7a87f70e4b5f152abafad1881482a9fa
  - 6f88ae29e80a5561dc28d54125bef807fa5211ec
close_out_documentation_ref: 4fc7dbbdb98ff0f3473f2793a18793624e43ee60
publication_correction_evidence: 6f88ae29e80a5561dc28d54125bef807fa5211ec
```

## Feature close-out

**State:** Current. The exact implementation set above passed final validation and the Sol High feature review. Draft [PR #40](https://github.com/JG1995/fm-valuescout/pull/40) is open and remains unmerged. The native desktop workflow is now confirmed; Repowise freshness remains an advisory gap only.

## Follow-up

- Keep draft [PR #40](https://github.com/JG1995/fm-valuescout/pull/40) unmerged until it is ready for review and the required checks pass.
- Plan **CSV enrichment persistence and derived statistics** separately. Define save-scoped storage, provenance, retention, source replacement, memory-over-CSV precedence, and user-visible consumers before implementation.
- Add calculations for Moneyball statistics that are absent from the exports only in that follow-up; this feature remains parsing and non-mutating reconciliation only.
