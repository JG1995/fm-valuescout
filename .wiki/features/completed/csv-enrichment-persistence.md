# CSV Enrichment Persistence and Derived Statistics

## Intent

Persist supported Youth Tracker and Moneyball CSV data that the memory pipeline does not supply, while keeping memory-backed player identity and fields authoritative. Transport the implemented Moneyball statistic catalogue and calculations from the pinned legacy repository into this repository.

## Delivered behavior

- The Dashboard imports one supported Youth Tracker or Moneyball CSV into the active app save after a current memory snapshot is loaded.
- Matching uses exact numeric FM UID equality against the active save's current snapshot. Unknown UIDs are reported as skipped and never create players or enrichment rows.
- Youth Tracker imports store all-time career appearances, goals, assists, and international caps for matching UIDs. Academy uses those values for career columns, aggregates, and graduate status.
- Moneyball imports store the latest per-season asking price, starts, substitute appearances, minutes, and 138 canonical exported-or-derived statistics for matching UIDs. This feature adds no Moneyball analytics screen.
- Re-importing replaces the complete row for each included matching UID in that format. Omitted matching players keep their prior enrichment, and the other format remains unchanged.
- Enrichment survives current-snapshot replacement and app restart, remains isolated by app save, and is deleted with its owning save.
- The import result reports the detected format, parsed-player total, stored count, and skipped count. The selected path and file contents are not returned, retained, or displayed.
- Invalid, stale, or failed imports write nothing. Native picker cancellation remains a no-op.

## Final architecture

- Migration v17 adds `player_youth_career_stats` and `player_moneyball_stats`, each keyed by `(save_id, player_uid)` and referencing `saves(id) ON DELETE CASCADE`. The tables do not reference snapshot-owned `players`, so current-snapshot replacement preserves enrichment while save deletion removes it.
- Rust `features/csv_import` validates bounded regular UTF-8 CSV files, detects the established Youth Tracker and Moneyball dialects, preserves nulls, canonicalizes the Moneyball contract, and performs the import transaction. It captures the active save, current snapshot, and eligible UID set under a brief lock, parses outside the lock, revalidates the context in the write transaction, and upserts only matching rows.
- Moneyball persistence stores one validated JSON object with the exact 138 canonical performance keys from the pinned legacy schema. Exported values remain authoritative except where the pinned implementation defines a fallback; derived values preserve null and zero boundaries and never emit non-finite numbers.
- React `features/csv-import` owns the dialog-backed Dashboard import action, bounded result state, safe errors, and context-generation guards. It clears state on save or snapshot changes, never displays the selected path, and invalidates Academy only after a successful Youth Tracker import.
- Academy joins Youth career enrichment by save and UID after membership identity is established. Current memory-backed identity remains authoritative, while tracked unresolved members can retain previously verified career values. A player is a graduate with at least one reported career appearance; incomplete aggregates and Graduates views remain unavailable when required values are null.

## Important decisions

- Keep enrichment in save-scoped tables instead of snapshot-owned `players`; snapshot replacement must not erase imported values.
- Use exact numeric UIDs and never create a general player or overlay memory-owned identity, ability, contract, club, position, attribute, value, or foot data from CSV.
- Keep one latest row per save, format, and player, with per-player full-row replacement. Do not clear rows omitted from a later file.
- Store the large Moneyball contract as one canonical JSON object rather than 138 SQLite columns or one row per metric.
- Do not add season identity, import history, or Moneyball presentation. A future history feature must define an explicit season contract before adding historical rows.
- No ADR or debug report was required. The feature uses the existing Rust IPC, trust-boundary, SQLite ownership, and save-scoping decisions.

## Migration and operational implications

- Migration v17 upgrades the current v16 schema additively. It adds nullable Youth career fields, nullable Moneyball base fields, the canonical statistics JSON object, asking-price constraints, and row-level import timestamps.
- Parsing and canonicalization occur outside the SQLite mutex. Context revalidation and all matching writes occur atomically; parse, conversion, database, or stale-context failures leave prior enrichment unchanged.
- The native picker path crosses IPC only as an inbound argument. The path, raw rows, complete file, file name, and file hash are not logged, returned, or persisted.
- The bridge dump schema and bridge extraction remain unchanged. The memory bridge is not required for validation after a current snapshot exists.

## Validation

- `./scripts/dev format` passed: 236 files checked, no fixes applied.
- `./scripts/dev test` passed: 25 frontend test files and 253 tests.
- `./scripts/dev check` passed Biome, TypeScript, secretlint, Rust format and Clippy, and Rust tests: 329 passed, 0 failed, and 2 ignored.
- `./scripts/dev smoke` passed 21 Chromium tests outside the sandbox. Smoke uses the browser IPC and dialog stub; it does not prove native picker behavior, real WebView-to-Rust IPC, SQLite persistence, or file-system access.
- `git diff --check main...HEAD` passed. Repowise was unavailable, so no indexed change-risk or impacted-test evidence was used.
- The feature-complete Sol xhigh review cleared with Blocking **No**, no CRITICAL, HIGH, MEDIUM, or NITPICK findings, project fit **Conforms**, action **Skip**, and recommendation **Accept**.
- No real desktop native picker/import/restart flow was run. A real desktop Youth import followed by Academy refresh and app restart remains an unexercised integration risk. No live FM26 validation was run; the ledger makes live scanning optional and not a completion gate.
- Mutation testing was not run because `./scripts/dev mutate` remains unsupported; it was not treated as a pass.

## Exact implementation refs

The exact scope starts at base ref `872f4590df8be5b8e1d1217e940ec80aab1093a8` (`feat(import): preview supported FM CSV exports`, PR #40 content baseline). Planning commit `fd4fc86` is context only and is excluded. The final PR content commits are:

| Ref | Subject | Result |
| --- | --- | --- |
| `b1d196bc97254e99b161747f802e97152121d31f` | `feat(import): add save-scoped enrichment schema` | Accepted |
| `84fee83a1dd80f3fc60b8974afe7b12e9f47d985` | `feat(import): derive canonical Moneyball statistics` | Accepted; corrected the inherited 176-key plan count to the pinned 138-key contract |
| `66f0e6b14da8f611c840c2400f49ce93322ff56f` | `feat(import): persist matched CSV player enrichment` | Accepted |
| `96460383d60a77e6c32d8743b53365cba476ddea` | `feat(academy): use imported Youth career statistics` | Accepted |
| `cfbb9413d7d2bf264cafb990e0b9db5dfd733682` | `feat(import): import CSV enrichment from Dashboard` | Accepted |

## Delivery profiles

| Ref | Implementation profile | Review profile |
| --- | --- | --- |
| `b1d196b` | Terra xhigh | Sol xhigh |
| `84fee83` | Terra xhigh | Sol High |
| `66f0e6b` | Terra Max | Sol xhigh |
| `9646038` | Terra xhigh | Sol High |
| `cfbb941` | Terra xhigh | Sol High |

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/csv-enrichment-persistence
base_branch: main
provisional_pr_title: "feat(import): persist CSV player enrichment"
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
build_feature_loop_profile: terra_max
feature_close_out: current
feature_review_profile: sol_xhigh
feature_review_blocking: false
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
project_fit: conforms
feature_review_action: skip
feature_review_recommendation: accept
ci_repair_rounds: 0
base_ref: 872f4590df8be5b8e1d1217e940ec80aab1093a8
implementation_range: "base ref plus the five final-PR content refs listed below; planning fd4fc86 excluded"
implementation_refs:
  - b1d196bc97254e99b161747f802e97152121d31f
  - 84fee83a1dd80f3fc60b8974afe7b12e9f47d985
  - 66f0e6b14da8f611c840c2400f49ce93322ff56f
  - 96460383d60a77e6c32d8743b53365cba476ddea
  - cfbb9413d7d2bf264cafb990e0b9db5dfd733682
close_out_documentation_ref: Pending record
publication_correction_evidence: none
```

## Feature close-out

**State:** Current. The exact base and final-PR content set passed the recorded validation and the Sol xhigh feature review. The final PR is not published and is not merged. No real native desktop test was run: picker/WebView IPC and the assembled desktop Youth import, Academy refresh, and restart flow remain unexercised. Live FM26 validation was not required.

## Follow-up

- Publish the final PR only when the branch is intentionally handed to the GitHub publication workflow. Do not publish or merge it during documentation reconciliation.
- Revisit the [Moneyball history backlog item](../../BACKLOG.md) when season comparisons or trends become planned work. The current tables intentionally keep only the latest row per save, format, and player.
