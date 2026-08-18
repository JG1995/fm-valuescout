# Complete Player Position Familiarity

## Intent

Linear JAY-14 preserves the complete raw FM26 position-familiarity map from bridge extraction through snapshot persistence. Each consumer now applies its own explicit relevance rule instead of using a missing JSON key as a threshold.

## Delivered behavior

- Schema-v7 dumps contain exactly 15 canonical keys: `GK`, `SW`, `DL`, `DC`, `DR`, `DM`, `ML`, `MC`, `MR`, `AML`, `AMC`, `AMR`, `ST`, `WBL`, and `WBR`.
- Each newly scanned value is an exact integer from `0` through `20`, including zero, or `null` when unread or invalid. Snapshot ingest validates the complete map before mutation and stores it unchanged.
- Recorded positions use familiarity `> 0` in Search, Squad, Academy labels, profile pitch labels, and best-position selection. Playable positions use `>= 15` in profile summaries and goalkeeper mode. Optimizer eligibility separately requires IP suitability of at least 16 and OOP suitability of at least 12 for distinct lane positions.
- Potential projection selects natural positions with `>= max(15, strongest - 2)`. If no value reaches 15, it selects the first strongest positive value in canonical layout order. Complete maps therefore preserve equivalent sparse-map projection behavior.
- The profile pitch has all 15 slots, including `SW`. It displays positive familiarity without inventing an `SW` role. The existing current/potential IP/OOP summaries and concealment behavior remain unchanged.
- Existing schema-v6 snapshots remain readable as sparse legacy data. New ingestion rejects stale dump files and requires an updated bridge scan for complete familiarity.

## Final architecture

The C# bridge owns coverage-aware extraction of the canonical position bytes. Rust validates the schema-v7 object and persists it transactionally in `players.positions_json`. Consumer layers own their recorded, playable, natural-projection, and raw-metric interpretations. See [the dump contract](../../../bridge/DUMP_SCHEMA.md) and [Architecture](../../ARCHITECTURE.md).

No ADR or debug report was needed: this work versions an existing protocol and preserves the established bridge, validation, persistence, and consumer ownership boundaries.

## Validation

- `./scripts/dev format` passed.
- `./scripts/dev bridge-test` passed: 201 passed and 3 skipped of 204 total.
- Rust tests in `./scripts/dev check` passed: 411 passed and 2 ignored. The full check passed.
- `./scripts/dev smoke` passed: 36 of 36 scenarios.
- `git diff --check 3e77e7864c0ed980630cbc0948f512c1dfd19f95...425d80fd4a7c2ca5679a934c02a39a95379e67c4` passed.
- Full Vitest reported 431 passed and 2 failed. The failures are unrelated release-policy subprocess tests that received empty stdout; this feature does not change release-policy files.

The Sol xhigh feature review reported Blocking: No and no CRITICAL, HIGH, or MEDIUM findings. Its two documentation consistency NITPICKs are corrected by this reconciliation.

## Exact implementation refs

The final PR starts from `origin/main` at `3e77e7864c0ed980630cbc0948f512c1dfd19f95`. It has no earlier PRs or correction commits.

| Ref | Subject | Result |
| --- | --- | --- |
| `6706e07e6594848aaab8ce7734e932bfa04a5412` | `refactor(scoring): derive natural positions from familiarity` | Projection receives familiarity values and applies the natural-position rule. |
| `f9ab62232dfecb48e0412349844a6626339bb868` | `refactor(search): filter recorded position familiarity` | Search and Squad use positive integer familiarity; a JSON-boolean filter issue was corrected before commit. |
| `e04627f1dc6482167532cfc35c7e2a4bbc7308f1` | `feat(profile): display complete position familiarity` | Profile and Academy preserve nullable maps, show positive familiarity, and keep the playable threshold. |
| `425d80fd4a7c2ca5679a934c02a39a95379e67c4` | `feat(memory-read): emit complete position familiarity` | Bridge extraction, schema-v7 validation, exact persistence, fixtures, and protocol documentation. |
| `ea82d2fea0dbc92769c7d60edf6f05a28bbb9113` | `docs(memory-read): close complete position familiarity feature` | Documentation reconciliation for this completed record. |

## Final publication

```yaml
status: published
pr_status: draft
merge_status: not_merged
pr_ref: "#56"
merge_ref: "Not merged"
branch: feature/player-position-familiarity
base_branch: main
base_ref: 3e77e7864c0ed980630cbc0948f512c1dfd19f95
provisional_pr_title: "feat(memory-read): preserve complete player position familiarity"
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
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: documentation consistency corrected in close-out
correction_rounds: 0
ci_repair_rounds: 0
implementation_range: 3e77e7864c0ed980630cbc0948f512c1dfd19f95..425d80fd4a7c2ca5679a934c02a39a95379e67c4
exact_implementation_refs:
  - 6706e07e6594848aaab8ce7734e932bfa04a5412
  - f9ab62232dfecb48e0412349844a6626339bb868
  - e04627f1dc6482167532cfc35c7e2a4bbc7308f1
  - 425d80fd4a7c2ca5679a934c02a39a95379e67c4
documentation_commit: ea82d2fea0dbc92769c7d60edf6f05a28bbb9113
```

## Feature close-out

**State:** Current. The exact implementation set passed the recorded validation and feature review. The branch is local and unpublished; no PR or merge ref exists.

## Follow-up

- Install the schema-v7 bridge in a supported Windows FM26 environment, run Load Data, and inspect a representative dump and stored row for all 15 keys, positive secondary values, zero, and unread `null`.
- Confirm live Search, Squad, profile, Planner, optimizer, and Academy behavior. Include the exact optimizer boundaries: IP familiarity 16 is eligible and 15 is not; distinct OOP familiarity 12 is eligible and 11 is not.
- Open a persisted schema-v6 snapshot to prove sparse legacy reads end to end. The deterministic suites cover sparse fixtures, but no dedicated persisted-v6 snapshot regression or live FM run was recorded.
- Publish only through the GitHub publication workflow. Resolve the pending documentation ref from Git when this reconciliation is committed.
