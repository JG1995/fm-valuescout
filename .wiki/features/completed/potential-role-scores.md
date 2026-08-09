# Potential Role Scores

## Intent

Project how well a player can fit each existing tactical role after development from current ability (CA) to potential ability (PA), while keeping current scores as the measure of present fit. The feature uses a position-sensitive visible-attribute projection and the existing Rust role scorer. It does not add a meta score or a probability that a player reaches PA.

## Delivered behavior

- Rust projects nullable visible attributes from CA to PA with the adopted position profiles and age factors. It preserves nulls, applies positive-only growth, rounds and caps values at 20, averages recognized natural-position groups, uses the `ALL` fallback, and treats `PA <= CA` as identity.
- The player profile Roles tab shows current and potential scores for every role in the 68-role catalog. Overview derives independent **Best Role** and **Best Potential Role** summaries from those ordered rows. The Attributes tab shows visible `Current → Potential` values; Hidden and Personality remain current-only.
- The Squad Planner shows current and potential combined scores for each resolved assignment. Both values use the lane's selected IP/OOP roles and saved IP weight. Unresolved assignments remain unavailable, and outside-pool assignments retain their visible warning state.
- The existing `optimize_planner_depth` command accepts a validated `current` or `potential` basis. Current optimization reads persisted `player_role_scores`; potential optimization projects eligible players from configured sources once per candidate record in each team candidate set before the shared ranked allocation, exact matcher, foot rules, manual reservations, replacement, transaction, and rollback paths.
- Both optimizer bases retain the existing source, team-age, position-suitability, lane-rank, preferred-foot, team/string order, filled-lane, UID tie-break, and assignment-provenance rules.
- **Optimize squads** remains the current-score action. **Optimize by potential** is an explicit secondary action. The slot picker and candidate DTO remain current-score-only, and potential values are not persisted.
- Current and potential labels, badges, arrows, pending states, and unavailable em dashes remain visible and accessible. No additional Load Data scoring pass, migration, database table, IPC command, or frontend scoring formula was added.

## Final architecture

- `src-tauri/src/features/scoring/projection.rs` owns the pure projection service and `projection_profiles.rs` owns the pinned empirical profile data. The implementation retains provenance for the FMSuperScout source at commit [`0f270d39`](https://github.com/mavarobli/FMSuperScout/blob/0f270d39a9cdc850ddfe653710d4904f13709cb5/app/app.js#L2738-L2808) and the project permission record in [`superscout-permission.md`](../../notes/superscout-permission.md).
- `get_player` projects one player's visible attributes once, reuses that map for all potential role scores and the profile DTO, and returns current and potential role rows plus `potentialAttributes`. React derives both best-role summaries and only formats Rust-returned values.
- `get_planner_depth` projects each resolved assigned player once, scores only the lane's selected IP/OOP pair, and returns `combinedScore` and `potentialCombinedScore`. Missing inputs remain unavailable.
- `optimize_planner_depth(score_basis)` validates `current` or `potential` at the Rust command boundary. Current mode uses persisted role scores. Potential mode reads only configured source players, projects each candidate record once in each team candidate set, and passes the resulting lane-score vectors through the existing allocation and persistence pipeline. Slot candidates continue to use current scores.
- Potential values remain derived read-time data. TanStack Query caches unchanged IPC results; Load Data, snapshot replacement, and persisted current-score behavior remain unchanged.

## Important decisions

- Apply the upstream `mentalGrowthFactor` at the documented age thresholds, even though the pinned upstream helper is currently unused.
- Treat `PA <= CA` as identity so current and potential values remain comparable for players without remaining ability headroom.
- Keep projection and scoring in Rust, reuse the existing catalog and combination helpers, and derive Overview summaries in React without a second backend field or scoring path.
- Keep Hidden and Personality attributes current-only because the adopted projection has no contract for them.
- Extend one optimizer command with one validated basis and preserve one allocation implementation. Do not persist score basis or derived potential values.
- No ADR is required. The feature stays inside established Rust business-logic, IPC, Query-cache, transaction, and non-persistent derived-data boundaries.

## Migration and operational implications

- No database migration or persisted-data change is required. `player_role_scores` continues to store current ingest-time values only.
- Potential profile and assigned-cell values are recalculated on their bounded Rust reads. Potential optimization runs only after the user invokes the explicit action and remains source-scoped.
- The existing Query invalidation and optimizer transaction boundaries remain in place. A future projection formula change changes derived potential values without rewriting snapshots.

## Validation

- `./scripts/dev format` made no changes.
- `./scripts/dev test` passed 23 files and 210 tests.
- `./scripts/dev check` passed the frontend quality checks and Rust validation with 259 tests passed and 2 ignored.
- Escalated `./scripts/dev smoke` passed 18 Chromium tests.
- Feature-complete review used Sol High. Blocking was **No** with no CRITICAL, HIGH, MEDIUM, or NITPICK findings. Project fit conforms to repository guidance.
- Native Tauri inspection at 1280×800 and 1600×900 was unavailable in the Linux session. Representative full-club profile, Planner, and potential-optimizer timings were not captured. Repowise remained stale at `4ad07c4` after its index-only refresh. These are evidence gaps, not passed checks; the review recommends explicit developer acceptance at close-out.

## Exact implementation refs

The final PR range is `b9ff83bf160609bd02061c6d168c6ac55c02dcdd..b770fd29ae11479eeb04d16d4abb23159bb52df2`. Every ref in that range is retained here, including planning and feature-content commits:

| Ref | Subject |
| --- | --- |
| `ae1f5b891c230dc1f5a7aaa895c74cf97f9b23c7` | `docs(scoring): plan potential role scores` |
| `ce7c87aab36fa523793c96c6aee8935dcb026c4f` | `feat(scoring): project attributes to player potential` |
| `126bf76ce73c11a02b77cdc70d5f9d8637971949` | `feat(profile): show potential scores for every role` |
| `273b1117268cf3d5330c87b2cbc9b92b031820cf` | `feat(planner): show potential score for assigned roles` |
| `a5920fc4ddaa34ac53751d899f0d1e6e5e2fde8a` | `docs(scoring): expand potential role score plan` |
| `156037c25d6b52101a765ea59b61fdfd19961c13` | `feat(profile): show the best potential role` |
| `d78c24a4f9015b927faad18081206ff816443321` | `feat(profile): show projected visible attributes` |
| `b770fd29ae11479eeb04d16d4abb23159bb52df2` | `feat(planner): optimize squads by potential` |

### Delivery profiles

| Commit | Implementation profile | Review profile | Result |
| --- | --- | --- | --- |
| `ce7c87a` — Project visible attributes to player potential | Terra xhigh | Sol High | Accepted after one fix round |
| `126bf76` — Show potential scores for every profile role | Luna Max | Sol Medium | Accepted |
| `273b111` — Show potential score for assigned Planner roles | Terra xhigh | Sol High | Accepted |
| `156037c` — Show the best potential role on Overview | Luna Max | Sol Medium | Accepted |
| `d78c24a` — Show projected visible attributes | Luna Max | Sol Medium | Accepted |
| `b770fd2` — Optimize squads by potential | Terra xhigh | Sol High | Accepted after one fix round |

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/potential-role-scores
base_branch: main
provisional_pr_title: "feat(scoring): add potential role scores"
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
build_feature_loop_profile: terra_xhigh
feature_close_out: current
feature_review_profile: sol_high
feature_review_blocking: false
ci_repair_rounds: 0
implementation_range: b9ff83bf160609bd02061c6d168c6ac55c02dcdd..b770fd29ae11479eeb04d16d4abb23159bb52df2
implementation_refs:
  - ae1f5b891c230dc1f5a7aaa895c74cf97f9b23c7
  - ce7c87aab36fa523793c96c6aee8935dcb026c4f
  - 126bf76ce73c11a02b77cdc70d5f9d8637971949
  - 273b1117268cf3d5330c87b2cbc9b92b031820cf
  - a5920fc4ddaa34ac53751d899f0d1e6e5e2fde8a
  - 156037c25d6b52101a765ea59b61fdfd19961c13
  - d78c24a4f9015b927faad18081206ff816443321
  - b770fd29ae11479eeb04d16d4abb23159bb52df2
close_out_documentation_ref: Pending record
publication_correction_evidence: none
```

## Feature close-out

**State:** Current. Validation and feature review cleared the exact implementation set above. The final PR remains unpublished and unmerged. Native desktop inspection, representative timing evidence, and a clean Repowise index remain explicitly unvalidated.

## Follow-up

- Obtain populated native Tauri inspection at 1280×800 and 1600×900 when the supported Windows/FM runtime is available.
- Capture representative profile, populated-Planner, and potential-optimizer timings before considering persistence, batching, or other performance changes.
- Refresh Repowise from the exact feature head when the index tooling is available; treat direct source, tests, configuration, and Git evidence as authoritative while it remains stale.
