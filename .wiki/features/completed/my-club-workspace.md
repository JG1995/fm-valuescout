# My Club Workspace

## Intent

Give managed-club player and staff workflows one My Club entry without changing their data, persistence, scoring, or mutation owners.

## Delivered behavior

- **My Club** is the primary destination at `/my-club`. It contains URL-backed **Squad**, **Planner**, **Tactic**, **Staff**, and **Staff Shortlist** workspaces, with Squad as the default.
- The My Club header owns the single explicitly saved managed-club selector at `/my-club#managed-club`. It uses exact current-snapshot club options and retains missing-saved-club warnings and downstream invalidation.
- **Player Search** remains at `/search`. **Staff Search** remains at `/staff`, while managed-club Staff and the save-owned Staff Shortlist live in My Club. Staff Shortlist remains joined to current staff by UID and is not managed-club filtered.
- Legacy `/planner`, `/staff?view=my-staff`, `/staff?view=shortlist`, and `/settings#managed-club` links replace into their equivalent My Club locations. `/staff/$uid` remains a Staff Profile route.
- Workspace changes replace URL search state. Squad, Planner, Tactic, Staff, and Staff Shortlist remain mounted so their local drafts, layouts, and selections survive tab changes.

## Final architecture

- `src/app/routes/my-club.tsx` is the canonical route, validated URL-state owner, and cross-feature composition seam. It owns the five workspace views and independent Squad, Staff, and Staff Shortlist sort and filter state.
- `src/features/my-club/components/my-club-workspace-tabs.tsx` owns accessible five-tab navigation. Existing feature components retain their query, mutation, persistence, and table-layout ownership.
- `src/app/routes/planner.tsx` and exact legacy `/staff` workspace URLs use replace redirects. `src/app/routes/staff.tsx` composes Staff Search and the Staff Profile outlet only.
- Rust and SQLite ownership is unchanged. One save-scoped managed club still supplies exact current-snapshot cohorts to Squad, Planner, Academy, and managed-club Staff. No migration, IPC command, query key, stored table-layout identifier, or capability changed.

## Important decisions

- Use `/my-club` as the canonical route and preserve old links with replace redirects instead of duplicate route implementations.
- Keep Staff Shortlist save-owned and separate from the managed-club cohort.
- Keep internal `my-staff` scopes and persisted identifiers unchanged while using **Staff** in My Club's visible copy.
- No ADR was needed. The change stays within accepted router, query, app-layer, Rust, and persistence boundaries.

## Validation

- Focused route suites passed: My Club Squad (99 tests), Staff (22), and legacy club routes (7).
- `./scripts/dev check` passed, including Biome, TypeScript, secretlint, Rust format, Clippy, and 481 Rust tests with 2 ignored.
- `./scripts/dev smoke` passed 45 Chromium tests, including 1280×800 and 1600×900 coverage for the header, selector, tables, Planner, Tactic, Staff, and Staff Shortlist.
- `./scripts/dev test` passed 495 of 497 tests. Two unrelated release-script tests failed while parsing empty JSON output: `scripts/release-metadata.test.ts` and `scripts/release-publication-policy.test.ts`.
- The developer supplied a cleared feature-complete review. This record does not add a verdict beyond that state.
- Native Tauri/WebView routing, focus, and viewport behavior remain unverified in a desktop runtime. `./scripts/dev mutate` remains unsupported.

## Exact implementation refs

**Base:** `1430f5d6607cdaf75f924226680e472b2658c431`

| Ref | Subject | Role |
| --- | --- | --- |
| `4dad469c66648aef3decf3fa6fc307b7756ea3dd` | `docs(club): plan My Club workspace` | Planning record |
| `941e2cf2409c5de245526b84839738f9abd12019` | `feat(club): move squad planning into My Club` | Canonical route, navigation, and Planner compatibility redirect |
| `dbf7afb1aab04807ae9e7891b443894cae5608c9` | `feat(club): move managed club selection into My Club` | Selector ownership, recovery target, and Settings reduction |
| `177ae67893ee03c54f35b8dc8c60eb84bebb1ade` | `feat(club): move club staff views into My Club` | Five-tab shell, Staff Search separation, and Staff compatibility redirects |
| `7c08539d987e8468520de76b4ccfb566e060c4fa` | `chore(release): prepare v0.8.0` | Release preparation in PR #69 |
| `ae6f2cd54068330862d4f546ed04689f774d132f` | `feat(club): unify managed club workspaces (#69)` | Merged main commit |

## Final publication

```yaml
status: merged
pr_status: merged
merge_status: merged
pr_ref: "https://github.com/JG1995/fm-valuescout/pull/69"
merge_ref: ae6f2cd54068330862d4f546ed04689f774d132f
branch: feature/my-club-workspace
base_branch: main
base_ref: 1430f5d6607cdaf75f924226680e472b2658c431
publication_provider: GitHub
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
feature_close_out: current
feature_review_status: cleared_developer_supplied
feature_review_acceptance: developer_supplied_cleared
final_pr_commit_set:
  - 4dad469c66648aef3decf3fa6fc307b7756ea3dd
  - 941e2cf2409c5de245526b84839738f9abd12019
  - dbf7afb1aab04807ae9e7891b443894cae5608c9
  - 177ae67893ee03c54f35b8dc8c60eb84bebb1ade
  - 7c08539d987e8468520de76b4ccfb566e060c4fa
close_out_documentation_ref: a4b4046c694f193dcc501fa6e1cd20f0f7d06051
```

## Feature close-out

**State:** Current. The exact implementation set passed the recorded validation and the developer-supplied feature-complete review, then merged as PR #69. Documentation reconciliation is complete on this close-out branch. Native Tauri/WebView routing, focus, and viewport behavior remain unverified in a desktop runtime.

## Follow-up

- Run the native Tauri/WebView route, focus, and supported-viewport checks when a desktop runtime is available.
