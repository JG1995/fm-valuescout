# Optional Moneyball Analysis Views

## Intent

Turn supported Moneyball CSV exports into an optional current-snapshot analysis workspace without adding the same metric density to General Search or Player Profile.

## Delivered behavior

- A successful Moneyball CSV import matches only the active save's current-snapshot players. It atomically replaces that snapshot's complete cohort and calculates nullable 0–100 percentile scores from the matched rows. Older snapshot rows and the legacy quarantine remain stored but are not shown.
- Player Search and Player Profile each have General and Moneyball views. General remains the factory default. Settings can set one app-local default for both views, while a valid URL view overrides that preference.
- Moneyball Search shows only the scored current cohort in the existing virtualized table. It provides all 138 performance metrics as raw-value columns, filters, and sorts, plus recruitment fields, playing-time context, separate saved layout, and Full CSV or complete filtered-cohort percentiles.
- Moneyball Player Profile shows asking price, playing-time context, and all 138 raw metrics with full-import percentile scores in eight accessible categories.
- Player Search owns Moneyball upload and replacement. My Club keeps only Youth Academy CSV upload.

## Final architecture

- Migration v30 adds nullable `percentiles_json` to snapshot-owned `player_moneyball_stats`. A null value marks a legacy current-format row that must be re-imported before current analysis can show it.
- Rust `features/moneyball` owns the exact metric contract, the lower-bound percentile engine, score direction, current-snapshot profile reads, and closed Search fields. It calculates full-import scores during import and filtered-cohort scores before the virtual table requests its bounded page.
- React owns Moneyball presentation metadata, raw-plus-score rendering, separate General and Moneyball table layouts, route view state, and the app-local Zustand preference. The WebView never receives an unbounded cohort for percentile calculation.

## Important decisions

- Keep current-snapshot analysis separate from future timeline and historical views.
- Preserve null values and exclude them from each metric population. Do not coerce them to zero.
- Keep percentile filters and percentile sorting out of scope. Search always filters and sorts raw values.
- Keep JAY-20 composite Moneyball role scoring separate from this feature.
- No ADR or debug report was needed. The work extends established snapshot, Rust/SQLite, route, and presentation-state boundaries.

## Validation and limitations

- `./scripts/dev format` completed without unintended changes.
- `./scripts/dev test` passed 53 files and 526 tests.
- `./scripts/dev check` passed, including Biome, TypeScript, secretlint, Rust formatting, Clippy, and 507 Rust tests with 2 ignored.
- `./scripts/dev smoke` passed 47 Playwright tests, including a Moneyball fixture with more than 100 rows.
- `git diff --check` passed for the final feature set.
- The Sol xhigh feature review used one correction round. The final verdict had Blocking **No**, no CRITICAL, HIGH, MEDIUM, or NITPICK findings, project fit **Conforms**, and recommendation **Accept**.
- Native Tauri/WebView behavior, real SQLite upgrade and restart persistence, upload picker/drop behavior, and representative near-1,000-row performance remain manual validation gaps. Browser stubs do not prove those paths.
- `./scripts/dev mutate` remains unsupported and was not reported as passed.

## Exact implementation refs

**Base:** `8a6b93c3c08177f268919df7dd743862fa64542c`

| Ref | Subject | Role |
| --- | --- | --- |
| `fc6f927` | `docs(plan): add Moneyball analysis feature plan` | Planning record |
| `2a3a99b4a429a940d9cbf1883bb1553d936f6151` | `feat(import): persist Moneyball percentile cohorts` | Schema v30, matched-cohort scoring, and atomic replacement |
| `813e3c72f5025f692731ccb0202c0092a298269b` | `feat(profile): add Moneyball analysis view` | Current-snapshot profile query and score-aware profile view |
| `5c8b22607059f2f9f480dc4a274c17539af18b35` | `feat(search): query Moneyball player cohorts` | Closed current-cohort Search and filtered-percentile query path |
| `ff02931de19288c9221d6cfda8ed20806405be9e` | `feat(search): add Moneyball search view` | Search UI, upload ownership, virtual table, and comparison pools |
| `77d1e07d0fa7202ecb058086a99b0ad65d9c0ec2` | `feat(settings): choose default player analysis view` | Shared persisted default and URL precedence |
| `53b5354508dc95f61cb3c6297b5fe7557ceabf6f` | `docs(moneyball): record implementation completion` | Active-ledger implementation completion record |
| `b3475ea23f95e2c262c0d427ba7e01126e18309e` | `fix(search): restore Moneyball recruitment fields` | Restored Parent club and Preferred foot as closed Moneyball Search fields and tested the exact recruitment/default-column contract |

The final feature review covered the five implementation commits and correction `b3475ea23f95e2c262c0d427ba7e01126e18309e`. Documentation reconciliation is `Pending record` until this close-out change is committed.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/moneyball-views
base_branch: main
base_ref: 8a6b93c3c08177f268919df7dd743862fa64542c
provisional_pr_title: "feat(moneyball): add optional analysis views"
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
earlier_prs: none
feature_close_out: current
feature_review_profile: sol_xhigh
feature_review_blocking: false
feature_review_recommendation: accept
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
project_fit: conforms
feature_review_action: skip
feature_review_correction_rounds: 1
ci_repair_rounds: 0
implementation_range: "8a6b93c3c08177f268919df7dd743862fa64542c..b3475ea23f95e2c262c0d427ba7e01126e18309e"
feature_review_scope: "2a3a99b4a429a940d9cbf1883bb1553d936f6151, 813e3c72f5025f692731ccb0202c0092a298269b, 5c8b22607059f2f9f480dc4a274c17539af18b35, ff02931de19288c9221d6cfda8ed20806405be9e, 77d1e07d0fa7202ecb058086a99b0ad65d9c0ec2, b3475ea23f95e2c262c0d427ba7e01126e18309e"
final_pr_commit_set:
  - fc6f927
  - 2a3a99b4a429a940d9cbf1883bb1553d936f6151
  - 813e3c72f5025f692731ccb0202c0092a298269b
  - 5c8b22607059f2f9f480dc4a274c17539af18b35
  - ff02931de19288c9221d6cfda8ed20806405be9e
  - 77d1e07d0fa7202ecb058086a99b0ad65d9c0ec2
  - 53b5354508dc95f61cb3c6297b5fe7557ceabf6f
  - b3475ea23f95e2c262c0d427ba7e01126e18309e
correction_ref: b3475ea23f95e2c262c0d427ba7e01126e18309e
correction_summary: "Restored Parent club and Preferred foot to the closed Moneyball recruitment catalog."
close_out_documentation_ref: Pending record
publication_correction_evidence: b3475ea23f95e2c262c0d427ba7e01126e18309e
```

## Feature close-out

**State:** Current. The exact corrected set passed final validation, the Sol xhigh feature review, and documentation reconciliation. The final PR remains local, unpublished, and unmerged.

## Follow-up

- Publish only through the GitHub publication workflow when the branch is ready for review. Resolve the pending documentation ref from Git after this reconciliation is committed.
- Run native Tauri/WebView checks at 1280×800 and 1600×900. Include tabs, table containment, filter editor, column controls, upload picker/drop, replacement feedback, focus restoration, restart persistence, and near-1,000-row import and filtered-search timing.
- Keep timeline/history work and composite role scoring in separately planned features.
