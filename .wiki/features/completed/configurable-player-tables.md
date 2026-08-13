# Configurable Player Tables

## Status

Completed — documentation reconciled; ready for publication

## Intent

Give Search and Squad one consistent, full-height player-table workspace. The feature combines staged metric filters, bounded virtual paging, sortable and configurable columns, on-demand potential role scores, and offline nationality flags.

## Delivered behavior

- Search and Squad render the shared full-height virtual player table. Each table owns one vertical scroll region, keeps fixed-height rows, requests bounded IPC pages as the virtual range moves, and has no Previous or Next controls. Row click, Enter, and the existing ArrowUp/ArrowDown focus path open profiles across page boundaries.
- Search keeps applied filters, filter-combine mode, sort field, and sort direction in validated URL state. Squad keeps workspace, sort field, and sort direction in URL state. Browser back and profile navigation therefore restore the applied table state.
- The Search filter editor copies applied rules into a local draft when it opens. Field, operator, value, add/remove, and combine changes stay query-silent until **Done** applies one complete draft. Cancel, close, Escape, and backdrop dismissal discard the draft. Applying a filter adds its metric column once when needed; filters and columns remain independent after that.
- One shared metric catalog and picker covers identity, club and contract, ability and reputation, visible and hidden attributes, personality, position suitability, current role scores, and potential role scores. Position displays a stable strongest-first list. Search and Squad can sort and display every validated sortable metric; Search retains its filter operators and Squad retains exact club-family membership.
- Search and Squad persist independent visible metric IDs, column order, and clamped widths in Zustand. Header click sorts. Right-click, Context Menu, and Shift+F10 open a keyboard-accessible menu with Move left, Move right, Add column, and Remove column. Pointer and keyboard resize handles preserve fixed `<colgroup>` widths, and at least one column remains visible. Reordering preserves widths, sorting, request identity, and the virtual window.
- Rust validates requested metric IDs before SQL construction and returns typed dynamic values through the Search and Squad DTOs. A potential field used only for display materializes the requested page; a potential filter or sort materializes the complete relevant current-snapshot cohort before count or ordering runs.
- Migration v21 adds the versioned, nullable `player_potential_role_scores` cache. Rows are sparse by requested role and store unknown results, stale model versions are replaced lazily, snapshot/player deletion cascades rows, and a successful supported player boost invalidates the changed player's rows in the same transaction. The cache is derived table data, not a new source of truth.
- Nationality cells render every stored nationality in order with bundled SVG flags. The explicit FM-name map covers the representative snapshot, including England, Scotland, Wales, Northern Ireland, and FM spellings such as `China PR`, `Ivory Coast`, `South Korea`, `Türkiye`, and `The Gambia`. `Zanzibar` uses a checked-in public-domain SVG because the package has no matching flag. Unknown future values remain truthful text, and empty arrays render `—`.

## Final architecture

- `src/components/player-table/virtualized-player-table.tsx` owns the shared semantic table shell, fixed row height, one vertical scroll owner, TanStack Virtual range, bounded page-window queries, roving row focus, whole-row activation, and horizontal overflow. `player-table-header.tsx` owns sorting, accessible context-menu actions, metric selection, and pointer/keyboard resizing. Search and Squad supply adapters for route state, query options, and cells.
- `src/utils/player-metrics.ts` is the frontend presentation catalog. `src-tauri/src/features/player_metrics/resolver.rs` independently validates the same metric families, builds trusted SQL expressions, and decodes dynamic values. Search owns filter AST compilation; Planner owns Squad club-family membership and its dynamic metric query. The WebView never selects raw SQL fields or roles.
- `src/stores/use-player-table-store.ts` persists versioned `search` and `squad` layouts only. URL state remains the source of truth for applied filters and sorting; Zustand stores visible columns, order, and widths; TanStack Query stores IPC results.
- Rust potential materialization lives in `features/player_metrics/potential_cache.rs`. Display-only requests use page UIDs. Global potential filters and sorts complete the current Search or Squad cohort in bounded resumable transactions before SQLite evaluates it. Profile and Planner potential reads remain direct projection paths outside this table cache.
- `players.nationalities_json` remains an ordered FM-name array. `NationalityCell` maps names to bundled assets without network access and renders the original value when no explicit map entry exists.

## Important decisions

- [ADR-0019 — Lazy persistent potential role-score cache](../../decisions/0019-lazy-potential-role-score-cache.md) records the additive v21 schema, sparse role/version rows, page-versus-cohort materialization, nullable-result reuse, and invalidation rules.
- Applied filters and sort stay URL-backed. Filter-editor changes remain local until Done; table preferences stay per-table and persisted in Zustand because they are client preferences, not shareable query state.
- Column reordering is menu-only. Assembled Tauri/WebView inspection continued to report a blocked cursor for native drag targets after a drop-handshake correction, while Chromium automation passed. The developer accepted Move left and Move right as the supported reorder path and removed direct drag-and-drop behavior and synthetic tests.
- Nationality rendering uses explicit FM-name aliases and a truthful text fallback. The one bundled Zanzibar asset is an approved exception to the package mapping, not a country substitution.

## Migration and operational implications

- Migration v21 is additive. Existing snapshots receive no precomputed potential rows; the cache grows only for requested roles and can be deleted and rebuilt. Composite foreign keys cascade rows when a snapshot/player is deleted.
- A successful supported boost deletes the affected player's cache rows because CA or Determination can change the derived result. The live boost invalidation path and assembled Windows timing behavior still need direct Windows evidence.
- Read-only inspection of the representative database confirmed schema v21, 182,836 players, and two complete potential-role cohorts totaling 365,672 cache rows. This confirms the inspected data shape, not a performance guarantee.
- The accepted operational gaps are the missing recorded Windows cold/warm potential-cache timings (including live supported-boost invalidation evidence) and outstanding native Tauri/WebView manual checks. Chromium, Rust, and frontend tests do not replace those checks.

## Validation

- `./scripts/dev format` made no changes.
- `./scripts/dev test` passed 33 files and 341 tests.
- `./scripts/dev check` passed, including 395 Rust tests with 2 ignored.
- `./scripts/dev smoke` passed 36/36 Playwright tests.
- `git diff --check d15b854178d8...328ad54de5679d4f3e342070e229fd499f7e712a` was clean for the exact feature range.
- The final Sol xhigh feature review returned Blocking **No**, no findings at any severity, Recommendation **Accept**, and architecture/project fit **Conforms**.
- Repowise was unavailable because its stale refresh did not progress. `./scripts/dev bridge-test` was outside the affected path, and `./scripts/dev mutate` remains unsupported; neither is reported as passed.

## Exact implementation refs

No earlier PRs exist. The complete publication range is `d15b854178d8..328ad54de5679d4f3e342070e229fd499f7e712a`. It contains two supporting, pre-implementation commits followed by the 13 commits reviewed for feature completion. The supporting refs are not part of the feature-complete implementation review:

| Ref | Subject | Role |
| --- | --- | --- |
| `7d30e60181944ce61fc61df68adc549add063987` | `docs(tables): plan configurable player tables` | Initial feature planning ledger; supporting context |
| `000f8cb6d50a976e1ffacb6fa19e2facb52da830` | `chore(config): ignore local MCP configuration` | Repository configuration; supporting pre-implementation change |

The exact feature-complete implementation review scope starts immediately after `000f8cb6d50a976e1ffacb6fa19e2facb52da830` and contains the 13 refs below:

| Commit | Ref | Subject | Review / role |
| --- | --- | --- | --- |
| 1 | `1014999d6a45521648a1ba7da68852782e47b43d` | `feat(search): stage organized filter changes` | Sol Medium — Accept after one correction round |
| 2 | `b258df83f578a20b1114e6de68a087512a28929a` | `feat(scoring): cache potential table role scores` | Sol xhigh — Accept after one correction round |
| 3 | `ef85cd6e01cb6c205c6a7dbba56b5b1118307302` | `feat(tables): query selected player metrics` | Sol xhigh — Accept after one correction round |
| 4 | `7b16eefa8f04a145054308f6f461f899d66cd181` | `feat(tables): virtualize full-height player lists` | Sol High — initial implementation accepted after correction rounds |
| 4 correction | `1fa76b293e74b566270b026aebe3e50da6943622` | `fix(tables): contain virtualized player table scrolling` | Sol High re-review — Accept, no findings |
| 5 | `484d3a2078419c3ec62b4b79bdf624dbac924cd9` | `feat(tables): persist resizable column layouts` | Sol xhigh — Accept; P1 correction re-review clean |
| 6 | `861380f8bd23a4894977810e7e574eff6953a2c8` | `feat(tables): render nationality flags` | Sol High — Accept, no findings |
| 7 | `762b0416173a20970bdfdb86e61a15b646f013cc` | `fix(tables): correct shared table interactions` | Sol Medium — Accept after Chromium validation |
| 7 correction | `ae8392d258503ca1ae6310dcb5a26223ca780faf` | `fix(tables): dismiss column menus on outside press` | Sol Medium correction — Accept, no findings |
| 8 planning adjustment | `6f302a49a2e502d4fbb185226a4810c08fc1a353` | `docs(tables): plan column reordering` | Planning adjustment retained in final PR set |
| 8 | `2b776892132b7ea774e8780e7457df718a18f48c` | `feat(tables): reorder visible columns` | Sol High — Accept |
| 8 attempted correction | `b71b24dcaa812bd13c3c5c805bd1b5251f43f66b` | `fix(tables): accept native column drops` | Native-drop correction; superseded by menu-only boundary |
| 8 final correction | `328ad54de5679d4f3e342070e229fd499f7e712a` | `fix(tables): remove unsupported column dragging` | Sol High re-review — Accept, no findings |

No correction commit was required after the feature-complete review. The documentation reconciliation ref is **Pending record** until this documentation-only change is checkpointed.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/configurable-player-tables
base_branch: main
base_ref: d15b854178d8
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
earlier_prs: none
build_feature_loop_profile: terra_max
feature_close_out: current
feature_review_profile: sol_xhigh
feature_review_blocking: false
feature_review_recommendation: accept
feature_review_architecture_fit: conforms
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
ci_repair_attempts: 0
publication_evidence: not_published/not_merged
publication_range: "d15b854178d8..328ad54de5679d4f3e342070e229fd499f7e712a"
feature_review_scope: "13 implementation commits after 000f8cb6d50a976e1ffacb6fa19e2facb52da830"
final_pr_commit_set:
  - 7d30e60181944ce61fc61df68adc549add063987
  - 000f8cb6d50a976e1ffacb6fa19e2facb52da830
  - 1014999d6a45521648a1ba7da68852782e47b43d
  - b258df83f578a20b1114e6de68a087512a28929a
  - ef85cd6e01cb6c205c6a7dbba56b5b1118307302
  - 7b16eefa8f04a145054308f6f461f899d66cd181
  - 1fa76b293e74b566270b026aebe3e50da6943622
  - 484d3a2078419c3ec62b4b79bdf624dbac924cd9
  - 861380f8bd23a4894977810e7e574eff6953a2c8
  - 762b0416173a20970bdfdb86e61a15b646f013cc
  - ae8392d258503ca1ae6310dcb5a26223ca780faf
  - 6f302a49a2e502d4fbb185226a4810c08fc1a353
  - 2b776892132b7ea774e8780e7457df718a18f48c
  - b71b24dcaa812bd13c3c5c805bd1b5251f43f66b
  - 328ad54de5679d4f3e342070e229fd499f7e712a
close_out_documentation_ref: Pending record
```

## Feature close-out

**State:** Current. The exact implementation set passed final validation and the Sol xhigh feature review. This branch remains local and unpublished; no PR or merge ref exists. The accepted Windows timing/live-invalidation and native Tauri/WebView manual-check gaps remain explicit above. Publication can resume from the machine-readable block without reopening implementation scope.

## Follow-up

- Publish the branch through the GitHub publication workflow when it is intentionally handed off. Preserve the exact final PR commit set and update publication refs only after GitHub evidence exists.
- On Windows with a representative current snapshot, record cold and warm Search/Squad potential-cache timings, request multiple potential roles together, verify warm reads do not add or rewrite rows, and exercise successful supported-player boost invalidation.
- Run the native Tauri/WebView visual and interaction checks at 1280×800 and 1600×900, including full-height scrolling, horizontal overflow, keyboard menus/resizing/reordering, profile/back state, and nationality hover labels.
