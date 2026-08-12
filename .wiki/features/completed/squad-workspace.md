# Squad Workspace

## Status

Completed — documentation reconciled; ready for publication

## Intent

Turn the Planner surface into a broader Squad workspace. The feature adds a club-scoped player overview, keeps the depth planner and tactic editor as separate pages, moves Club Setup to the Dashboard, adds explicit squad CSV imports, and applies the two approved player-development actions across the configured squad.

## Delivered behavior

- The primary navigation label is **Squad**. The stable `/planner` route contains URL-backed **Squad**, **Planner**, and **Tactic** workspaces, with Squad first and default. Planner depth and Tactic remain separate mounted pages, and Club Setup is available from the Dashboard at `/#club-setup`.
- Squad Overview shows only current-snapshot players whose exact current club belongs to the configured club family. It has no filters, uses the fixed Name, Age / DOB, Nationality, Club, Division, CA, PA, and Value columns, sorts from clickable headers, and links each player name to `/players/$uid`. Missing setup and snapshot states retain recovery links.
- **Upload Moneyball CSV** and **Upload Youth Academy CSV** open accessible single-file drag-and-drop or browse modals. Each modal enforces its selected format before persistence and reports the existing total, stored, and skipped counts. The Dashboard auto-detect importer remains available.
- **Boost all CA** uses the closed CA action sequentially. Players aged 20 or younger receive +5, players aged 21 through 28 receive +10, and players aged 29 or older receive no boost. PA and 200 caps remain in force, and ineligible players do not reach the bridge.
- **Make all Wonderkids** uses the closed Wonderkid Mentality action sequentially. Known Ambition, Professionalism, and Determination values at or below 10 receive inclusive random values from 11 through 20. Unknown and higher values remain unchanged.
- Both squad actions freeze the Rust-derived club cohort, prevent overlapping profile or squad actions, reconcile each verified player before the next request, and report truthful updated, skipped, and failed counts. Proven no-write player-local rejections continue; context loss or recovery-required bridge, verification, rollback, or reconciliation outcomes stop before another write and direct the user to Load Data. The affected snapshot keeps a recovery latch until a new effective snapshot is ingested.

## Final architecture

- `src/app/routes/planner.tsx` is the Squad composition root. It owns validated workspace state and composes the Squad table, CSV actions, Planner depth matrix, and Tactic editor without changing the stable route. Dashboard owns the existing Club Setup and auto-detect CSV panel.
- Rust owns the bounded Squad Overview read model, exact club-family membership, sortable pagination, expected CSV format enforcement, and the two squad-wide orchestration commands. The WebView sends closed actions only; it cannot choose player UIDs, increments, fields, or random targets.
- The C# bridge remains the only process-memory writer and continues to receive one action-specific request for one player at a time. Rust captures save and snapshot context, uses the shared player-boost gate, verifies live results, reconciles SQLite, and recomputes affected Determination role scores before continuing.
- Migration v20 adds `snapshots.player_boost_recovery_required`. A terminal boost uncertainty sets the flag and later profile or Squad boosts fail closed until Load Data establishes a new effective current snapshot. Existing Planner, tactic, save, snapshot, Academy, Search, profile, and bridge contracts remain unchanged outside the delivered behavior.

## Important decisions

- [ADR-0018 — Squad-wide action-specific player boosts](../../decisions/0018-squad-wide-player-boosts.md) records the sequential best-effort boundary, shared gate, recovery latch, corrected CA age rule, and rejected batch-write alternatives.
- Keep the bridge protocol action-specific and one-player. Do not add a general batch payload, arbitrary editor, parallel writes, undo, or WebView-supplied cohort or target values.
- Keep Dashboard Club Setup and its existing importer during the transition because Dashboard is not yet removed from the user-facing app.

## Migration and operational implications

- Migration v20 is additive and defaults existing snapshots to no recovery requirement. A snapshot marked for recovery is read-only for player boosts until a fresh Load Data run makes a new effective snapshot current.
- Squad actions can partially succeed before a later player fails. FM and SQLite are not one transaction; a recovery outcome preserves the observed counts and requires Load Data rather than claiming an all-or-nothing rollback.
- The Linux validation environment did not provide the .NET 6 SDK, so `./scripts/dev bridge-test` could not run. Native Tauri drop and dialog behavior, SQLite-file persistence, and live-FM integration were not run; browser IPC stubs and Rust tests do not prove those paths.

## Validation

- `./scripts/dev format` made no changes.
- `./scripts/dev test` passed 27 files and 293 tests.
- `./scripts/dev check` passed with 375 Rust tests and 2 ignored tests.
- `./scripts/dev smoke` passed 29 Playwright tests.
- `git diff --check 508772d...8e143eb` passed.
- The final Sol xhigh feature review accepted the exact implementation set with no findings.
- `./scripts/dev bridge-test` was blocked because the .NET 6 SDK is unavailable. `./scripts/dev mutate` remains unsupported and was not reported as passed.

## Exact implementation refs

Planning context is retained for history but excluded from the implementation set:

| Ref | Subject | Role |
| --- | --- | --- |
| `fab15ed9ddede3f44a46141465521d2b3542e156` | `docs(squad): plan squad workspace feature` | Planning context; excluded from implementation |

The implementation range is `fab15ed9ddede3f44a46141465521d2b3542e156..8e143eb46c92f40cf1abb39c8ac2c8e09514e037`:

| Ref | Subject | Review |
| --- | --- | --- |
| `c6e702f3773c7e6fb8ce37d90d9f0acbea17872e` | `feat(squad): reorganize squad navigation` | Sol Medium — Accept |
| `48adddfaa3197adf37d8424f73bb089e7b7c5601` | `feat(squad): list configured club players` | Sol High — Accept |
| `87a4f00e1990bf6060255605fe4a1af4547bc08d` | `feat(import): add squad CSV import modals` | Sol High — Accept |
| `bfa4328b86c42707cc7a37c110a54c24370c77bb` | `fix(player): correct CA boost age eligibility` | Sol High — Accept |
| `fceee890ca08e3a3843fe3f22f8aa31f198b4da5` | `feat(squad): boost squad current ability` | Sol xhigh — Accept |
| `8e143eb46c92f40cf1abb39c8ac2c8e09514e037` | `feat(squad): apply Wonderkid Mentality to squad` | Sol xhigh — Accept |

## Delivery profiles

| Commit | Implementation profile | Review profile |
| --- | --- | --- |
| `c6e702f` | Luna Max | Sol Medium |
| `48adddf` | Terra xhigh | Sol High |
| `87a4f00` | Terra xhigh | Sol High |
| `bfa4328` | Luna Max | Sol High |
| `fceee89` | Terra Max | Sol xhigh |
| `8e143eb` | Terra xhigh | Sol xhigh |

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/squad-workspace
base_branch: main
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
implementation_range: "fab15ed9ddede3f44a46141465521d2b3542e156..8e143eb46c92f40cf1abb39c8ac2c8e09514e037"
planning_ref: fab15ed9ddede3f44a46141465521d2b3542e156
planning_ref_role: context_only
implementation_refs:
  - c6e702f3773c7e6fb8ce37d90d9f0acbea17872e
  - 48adddfaa3197adf37d8424f73bb089e7b7c5601
  - 87a4f00e1990bf6060255605fe4a1af4547bc08d
  - bfa4328b86c42707cc7a37c110a54c24370c77bb
  - fceee890ca08e3a3843fe3f22f8aa31f198b4da5
  - 8e143eb46c92f40cf1abb39c8ac2c8e09514e037
close_out_documentation_ref: Pending record
```

## Feature close-out

**State:** Current. The exact implementation set passed final validation and the Sol xhigh feature review. The branch is local; no push, PR publication, or merge occurred. The accepted native bridge, Tauri, SQLite-file, and live-FM validation gaps remain explicit above.

## Follow-up

- Publish the PR only through the GitHub publication workflow when the branch is intentionally handed off.
- Re-run the native import and supported FM26.3.2 squad-action flows when a Windows Tauri and live-FM environment is available. Keep browser IPC, Rust, and C# fake evidence separate from that manual proof.
