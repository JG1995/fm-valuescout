# Squad Boost Progress and Feedback

## Intent

Show truthful progress for both sequential squad-wide development actions. Keep their final feedback in a stable Squad overview region.

## Delivered behavior

- After confirmation, **Boost all CA** and **Make all Wonderkids** show an indeterminate preparing state. Once Rust captures the frozen cohort, the open Modal shows determinate `processed / total` progress.
- Rust sends an initial `0 / total` update and an update after each updated, skipped, or failed player. `processed` equals the final counters' sum. A player that causes a recovery-required stop is not counted as processed.
- The command-scoped progress channel is best-effort. A failed delivery does not interrupt, retry, or reclassify a bridge write or reconciliation result. The final command result remains authoritative.
- The latest action's normal result appears in one reserved Squad overview region as compact processed, updated, skipped, and failed feedback. It does not move the action header.
- A recovery-required result reports only observed outcomes, tells the user that processing stopped, requires Load Data, disables both actions, and focuses the shared feedback region after the Modal closes.
- An error remains in the open Modal. A new effective current snapshot clears prior Modal progress and final feedback.

## Final architecture

- Rust `features/player` owns the frozen cohort, terminal outcome counters, and `SquadPlayerBoostProgressDto`. The two closed squad commands receive one typed Tauri IPC channel per invocation and report immutable counter snapshots from the existing sequential loop.
- React creates the typed channel in each Squad API adapter. `planner.tsx` keeps mutation, current-snapshot guards, invalidation, latest-action selection, and recovery focus ownership.
- `squad-player-boost.tsx` owns confirmation-Modal progress. `SquadOverviewPanel` renders the Squad-specific reserved final-feedback region without changing shared `Panel` or `Modal` primitives.
- The feature does not change the bridge protocol, SQLite schema, migrations, write order, shared player-boost gate, recovery latch, or profile-level actions.

## Important decisions

- Keep progress within the active confirmation Modal and final feedback in one reserved Squad overview region. Do not add global events, polling, a notification store, persistence, or feedback history.
- Keep the existing Rust-owned sequential mutation boundary. [ADR-0018 — Squad-wide action-specific player boosts](../../decisions/0018-squad-wide-player-boosts.md) remains current and unchanged.

## Validation

- `./scripts/dev test src/app/routes/planner.test.tsx` passed 74 focused Planner tests, including both actions, progress, final feedback, recovery, focus, errors, and snapshot replacement.
- `./scripts/dev check` passed the configured gate, including 396 Rust tests with 2 ignored tests.
- `./scripts/dev smoke` passed 35 browser tests, including intermediate progress and stable action-header layout.
- `./scripts/dev bridge-test` was outside the changed path because the C# bridge did not change. `./scripts/dev mutate` remains unsupported and was not treated as a pass.
- The feature-complete Sol xhigh review reported Blocking **No**, no CRITICAL, HIGH, MEDIUM, or NITPICK findings, Project fit **Conforms**, Action **Skip**, and Recommendation **Accept**.
- Host execution did not include an assembled native Tauri/WebView channel run. Browser smoke uses an IPC stub; direct Rust and frontend command-contract tests cover both adapters. This is the accepted low-risk validation gap.
- Repowise was stale at `43f4de9` and its refresh stalled. Direct source, tests, configuration, and Git evidence replaced indexed evidence.

## Exact implementation refs

The feature branch starts from `origin/main` at `cbfeaa53d0ce491475f0a4e64823e68ce75b3b85`. The two behavioral commits were the feature-complete review scope. The later TODO bookkeeping commit is part of the final PR set but not behavioral implementation.

| Ref | Subject | Role |
| --- | --- | --- |
| `b656428b7acafec4a08fe53054a03f086e31f28e` | `feat(squad): stream boost progress` | Command-scoped Rust and WebView progress path; Sol Medium re-review accepted |
| `08f9f26c3e4c9195637866ea5fde0222efcbbb57` | `feat(squad): stabilize boost feedback` | Reserved latest-action feedback region; fresh Sol Medium review accepted |
| `d733575793a61d62f31b117f17a30075898ef264` | `docs(wiki): record squad boost feature` | Prior TODO bookkeeping; excluded from behavioral scope |

## Final publication

```yaml
status: merged; release_publication_pending
pr_status: merged
merge_status: merged
pr_ref: "https://github.com/JG1995/fm-valuescout/pull/53"
merge_ref: c2cb0be78eb0e3f849cea4d8bb61dfde3601f8ee
branch: feature/squad-boost-progress
base_branch: main
base_ref: cbfeaa53d0ce491475f0a4e64823e68ce75b3b85
provisional_pr_title: "feat(squad): add boost progress feedback"
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
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
implementation_range: "cbfeaa53d0ce491475f0a4e64823e68ce75b3b85..08f9f26c3e4c9195637866ea5fde0222efcbbb57"
publication_range: "cbfeaa53d0ce491475f0a4e64823e68ce75b3b85..d733575793a61d62f31b117f17a30075898ef264"
implementation_refs:
  - b656428b7acafec4a08fe53054a03f086e31f28e
  - 08f9f26c3e4c9195637866ea5fde0222efcbbb57
documentation_ref: d733575793a61d62f31b117f17a30075898ef264
final_pr_commit_set:
  - b656428b7acafec4a08fe53054a03f086e31f28e
  - 08f9f26c3e4c9195637866ea5fde0222efcbbb57
  - d733575793a61d62f31b117f17a30075898ef264
close_out_documentation_ref: Pending record
```

## Feature close-out

**State:** Merged and release prepared. The exact implementation set passed validation and feature review, then merged as PR #53. Version `0.2.0` and its feature changelog are prepared on `feature/release-squad-boost-progress`; only verified-main publication remains. The native Tauri/WebView and Repowise gaps remain recorded above.

## Follow-up

- Publish `feature/release-squad-boost-progress` only when its `0.2.0` release preparation is intentionally handed to the GitHub publication workflow.
- Run the assembled native Tauri/WebView channel path when a supported Windows environment is available.
