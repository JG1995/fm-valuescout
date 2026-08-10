# Player Development Boosts

## Intent

Add two guarded actions to a player profile that change a running FM26 save through the existing BepInEx bridge: **Boost CA** and **Wonderkid Mentality**. The feature keeps process-memory writes action-specific, binds each write to a successful live bridge scan and the active snapshot, verifies the live result, and reconciles that result into SQLite.

## Delivered behavior

- The player profile Overview tab shows a **Development boosts** panel with one primary **Boost CA** action and one secondary **Wonderkid Mentality** action.
- **Boost CA** derives the increment from snapshot age: +5 at age 21 or younger and +10 at age 22 or older. Rust caps the target at the lower of PA and 200, preserves PA, and permits another confirmed boost while headroom remains.
- **Wonderkid Mentality** independently rerolls known Ambition, Professionalism, and Determination values at or below 10 to inclusive random integers from 11 through 20. Unknown and already-high values remain unchanged.
- The panel derives previews from the loaded player, confirms each write, prevents duplicate submissions, restores focus, reports pending and phase-specific errors, and shows the verified values on success. The copy explains that FM may redistribute attributes over the following in-game days, sometimes up to one month.
- The WebView sends only a player UID and one of the two closed commands. It cannot supply an address, field, target value, age, increment, or random result.
- A successful verified result updates the captured current snapshot and any affected current role scores in one SQLite transaction. The route invalidates snapshot, search, player, Planner, and Academy query roots so downstream views read the reconciled row.
- A missing or stale snapshot provenance, absent live index, unsupported build, changed live value, timeout, or uncertain rollback fails closed and gives the user a Load Data or plugin-update recovery path. FM-success followed by SQLite failure remains an explicit partial outcome.

## Final architecture

- The C# BepInEx bridge owns process-memory access and writes. It keeps a private UID-to-location index only after a successful live full dump, binds it to that dump request ID, revalidates the live UID and expected values, writes typed byte or unsigned-16-bit fields, reads them back, and reports rollback state without exposing addresses.
- Protocol v1 and dump schema v6 remain compatible. The two closed boost operations use additive request and status fields, and the bridge advertises support only for the explicitly write-validated FM26.3.2 build. Scans and boosts share one operation gate; PSS-backed scan addresses are never write targets.
- Migration v16 adds nullable `snapshots.bridge_source_request_id`. Load Data captures the completed dump and request ID before ingest and writes the provenance in the same transaction. Existing snapshots remain readable but cannot authorize a boost until a fresh Load Data supplies provenance.
- Rust `features/memory_read` serializes the bridge request and waits without holding the SQLite mutex. Rust `features/player` derives eligibility from the active snapshot, submits the UID-only command, verifies that the captured save and snapshot are still current, reconciles CA and personality or attribute JSON, and recomputes that player's role scores when Determination changes.
- React `features/player-profile` owns the panel and accessible mutation states. `src/app/routes/players.$uid.tsx` owns cross-feature invalidation and clears mutation feedback when the player or snapshot context changes.

## Important decisions

- Keep the write surface to two action-specific operations. A general editor, arbitrary address API, direct numeric input, undo history, or additional fields remain out of scope.
- Keep the bridge as the only process-memory writer. Rust owns snapshot policy and persistence; React only previews and presents results.
- Reconcile the verified targeted result into the same snapshot instead of forcing a full Load Data scan after every action. A later Load Data remains the source of truth for ordinary FM progression and delayed CA redistribution.
- Fail closed for unknown or unvalidated FM builds and for missing live-scan provenance. Exact build 26.3.2 is the only write-validated build recorded here.
- [ADR-0017 — Action-specific FM26 player boosts](../../decisions/0017-action-specific-fm26-player-boosts.md) records the durable write boundary and rejected alternatives.

## Migration and operational implications

- Migration v16 is additive. Existing databases receive null bridge provenance and retain their current snapshots; those rows are read-only for boosts until a new Load Data run.
- A successful bridge operation changes the running FM process before local reconciliation. If the SQLite transaction cannot commit, the app does not claim to undo FM; it reports the partial outcome and requires Load Data.
- The supported runtime remains Windows Steam FM26 with the installed BepInEx bridge. The bridge unit suite and app tests run without a live FM process; real integrated FM validation remains a separate manual gate.

## Validation

- `./scripts/dev format` processed 229 files with no fixes.
- `./scripts/dev bridge-test` passed 200 tests with 3 expected Windows-only skips.
- `./scripts/dev test` passed 23 frontend test files and 232 tests.
- `./scripts/dev check` passed the repository gate, including 289 Rust tests and 2 documented ignores.
- `./scripts/dev smoke` passed 19 Playwright tests.
- `git diff --check` passed.
- Feature-complete review used Sol xhigh after one correction round. Blocking was **No**; CRITICAL, HIGH, MEDIUM, and NITPICK findings were all **None**. Project fit was **Conforms**, action was **Skip**, and the recommendation was **Accept**.
- The initial feature review found three MEDIUM issues: preserved live bridge-index availability after a failed scan, stale mutation feedback across player or save navigation, and incomplete redistribution-timing copy. Commit `8052c90880ab4bd3354e5294d370591905e6f26d` corrected all three; follow-up review retained no findings.
- Fresh integrated live-FM validation of the final Rust, SQLite, cache, and UI path was explicitly skipped and accepted by the developer because this is a work machine. The controlled FM26.3.2 bridge proof recorded for the prior PR remains bridge evidence, but it does not independently prove the final integrated path. This accepted gap is the remaining validation risk.
- Mutation testing remains unsupported by `./scripts/dev mutate` and was not treated as a pass.

## Exact implementation refs

The bridge foundation merged in PR 37 at `1f4c57754de3585fe71cfc1830963601a8da296c`.

The final PR remains unpublished and unmerged. Its content commits and reviewed correction are retained here:

| Ref | Subject | Result |
| --- | --- | --- |
| `9f0b5983d9c2b2abc960f066f264d248148f4c96` | `feat(snapshot): bind snapshots to bridge scans` | Accepted after request/dump correlation and transaction-boundary correction |
| `bc5678c906f4f7d429ae38114911450a3cd0c40c` | `feat(player): persist verified player boosts` | Accepted after one ledger lifecycle correction |
| `77998deeae6f44ace64c425ba6e88bf68211eaa8` | `feat(profile): add CA boost action` | Accepted after one architecture-summary correction |
| `62a23afd85a22b2592ad953ffbfb775d36ffce1c` | `feat(profile): add Wonderkid Mentality action` | Accepted after two correction rounds |
| `8052c90880ab4bd3354e5294d370591905e6f26d` | `fix(player): align boosts with the active context` | Reviewed correction; all follow-up findings cleared |

## Delivery profiles

| Ref | Implementation profile | Review profile |
| --- | --- | --- |
| `9f0b598` | Terra xhigh | Sol xhigh |
| `bc5678c` | Terra Max | Sol xhigh |
| `77998de` | Luna Max | Sol High |
| `62a23af` | Luna Max | Sol High |

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/player-development-boosts
base_branch: main
provisional_pr_title: "feat(profile): add player development boosts"
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
prior_pr_ref: "https://github.com/JG1995/fm-valuescout/pull/37"
prior_pr_merge_ref: 1f4c57754de3585fe71cfc1830963601a8da296c
implementation_range: "1f4c57754de3585fe71cfc1830963601a8da296c..8052c90880ab4bd3354e5294d370591905e6f26d"
implementation_refs:
  - 9f0b5983d9c2b2abc960f066f264d248148f4c96
  - bc5678c906f4f7d429ae38114911450a3cd0c40c
  - 77998deeae6f44ace64c425ba6e88bf68211eaa8
  - 62a23afd85a22b2592ad953ffbfb775d36ffce1c
correction_ref: 8052c90880ab4bd3354e5294d370591905e6f26d
correction_summary: "Preserved boost capability after a failed scan, cleared stale feedback across player and snapshot navigation, and aligned redistribution timing copy."
close_out_documentation_ref: Pending record
publication_correction_evidence: 8052c90880ab4bd3354e5294d370591905e6f26d
```

## Feature close-out

**State:** Current. The exact corrected set above passed final validation and feature review. The final PR remains unpublished and unmerged by design. The accepted integrated live-FM validation gap is recorded above; the prior controlled FM26.3.2 bridge proof does not replace it.

## Follow-up

- Publish the final PR only when the branch is intentionally handed to the GitHub publication workflow. Do not publish or merge it during documentation reconciliation.
- Re-run one fresh integrated profile flow against a supported Windows FM26 session when the work machine is available. Keep the existing bridge proof as supporting evidence, not as proof of the Rust/SQLite/cache/UI path.
- Keep additional FM edit actions behind a new product decision; this record does not widen the two-action boundary.
