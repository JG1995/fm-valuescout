# Optional Planner Teams

## Status

Completed — draft PR #66 open for review

## Delivered behavior

- Each app save can keep one to three fixed Planner categories: Senior, Reserves, and Youth. The user can set a distinct display name for every available category.
- A new or migrated save starts with all three categories and their canonical names. Team availability and display names are save-scoped. Loading, switching, refreshing, or promoting a snapshot does not recreate a removed category or copy settings from another save.
- The Planner's **Manage teams** Modal validates non-empty, unique display names of at most 40 Unicode scalar values. It cannot add a fourth category or remove the final category.
- Removing an empty category updates Planner immediately. Removing a populated category requires confirmation that names the affected display name and assignment count, then removes its strings and assignments atomically. Re-adding a category creates one empty string with its canonical name.
- Planner tabs, grouped matrix headings, picker locations, action feedback, clear-all confirmation, and accessible names use persisted display names. Stable `senior`, `reserves`, and `youth` values remain the only internal category identities.
- Depth reads, picker and string mutations, and current- and potential-score optimization operate only on available categories. Removing the selected category selects a remaining category and moves focus to its tab or **Manage teams** when the grouped layout has no tabs.

## Final architecture

- Migration v28 adds save-owned `planner_teams(save_id, team, display_name)`. Row presence is availability. It backfills canonical rows for existing saves without changing Planner strings, assignments, tactics, club-family sources, snapshots, Academy data, or staff shortlists.
- Rust `features/planner::teams` owns initialization, validation, canonical ordering, and one transactional complete-configuration replacement. `depth`, picker, string, and optimizer paths use its available-team contract rather than assuming all three categories exist.
- `get_planner_depth` returns available teams in canonical order with display names. React keeps display names in presentation and sends stable category values through IPC. The depth matrix replaces the returned cache, invalidates slot candidates, and closes stale picker or string state after a successful team save.
- No ADR was needed. The save-owned table and transaction follow the existing Planner persistence boundary. No debug report was needed because regression tests and this record explain the behavior.

## Validation and limitations

- `./scripts/dev format` was idempotent.
- `./scripts/dev test` passed 45 files and 498 tests.
- `./scripts/dev check` passed, including 482 Rust tests with 2 ignored.
- `./scripts/dev smoke` passed 46 Playwright tests.
- The final Sol xhigh feature review returned Blocking **No**; no correction was required.
- Chromium smoke covers the stateful Planner-management flow but does not prove the native Tauri/WebView keyboard path or minimum-window behavior. That native pass was unavailable at close-out.
- Repowise change-risk and impacted-test evidence is unavailable: the index was stale and refresh did not complete. Direct source, test, Git, and review evidence remain the basis for this record.

## Exact implementation refs

**Feature range:** `e713e2449059b162c392407c3b042cec3196e067..ac036c58a8a5704277537d24e94c6df29acb91b0`

| Ref | Subject | Role |
| --- | --- | --- |
| `abaa8bb3e5af1d319e4252a27188e5c0fed75dbf` | `docs(planner): plan configurable squad teams` | Planning record |
| `945025cb1cacfcf11e51d9d6e28de4efd1b908d6` | `feat(planner): persist save-scoped team settings` | v28 persistence, availability guards, transactional replacement, cleanup, and optimizer integration |
| `aa67ab79f4a070aa7a723a0fbc4a39c8c33927b6` | `feat(planner): render configured squad teams` | Dynamic display-name rendering and available-team keyboard navigation |
| `ac036c58a8a5704277537d24e94c6df29acb91b0` | `feat(planner): add squad team management` | Management Modal, confirmation, cache and focus reconciliation, IPC doubles, and browser smoke |

No correction commit was required after the feature-complete review. Documentation reconciliation committed as `ad62448`.

## Final publication

```yaml
status: published
pr_status: draft
merge_status: not_merged
pr_ref: "https://github.com/JG1995/fm-valuescout/pull/66"
merge_ref: "Not merged"
branch: feature/optional-planner-teams
base_branch: main
base_ref: e713e2449059b162c392407c3b042cec3196e067
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
ci_repair_attempts: 0
publication_evidence: draft_pr_66/not_merged
publication_range: "e713e2449059b162c392407c3b042cec3196e067..ac036c58a8a5704277537d24e94c6df29acb91b0"
feature_review_scope: "945025cb1cacfcf11e51d9d6e28de4efd1b908d6, aa67ab79f4a070aa7a723a0fbc4a39c8c33927b6, ac036c58a8a5704277537d24e94c6df29acb91b0"
final_pr_commit_set:
  - abaa8bb3e5af1d319e4252a27188e5c0fed75dbf
  - 945025cb1cacfcf11e51d9d6e28de4efd1b908d6
  - aa67ab79f4a070aa7a723a0fbc4a39c8c33927b6
  - ac036c58a8a5704277537d24e94c6df29acb91b0
close_out_documentation_ref: ad62448f4b51ddf452d98848e487f61e701ef8c6
```

## Feature close-out

**State:** Current. The exact implementation set passed final validation, the Sol xhigh feature review, and documentation reconciliation. Draft PR [#66](https://github.com/JG1995/fm-valuescout/pull/66) is open from `feature/optional-planner-teams`; no merge ref exists.

## Follow-up

- Publish the branch through the GitHub publication workflow when it is intentionally handed off. Update publication refs only after GitHub evidence exists.
- Run native Tauri/WebView keyboard and minimum-window checks for team management and selection fallback when a desktop environment is available.
