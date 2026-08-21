# Planner Best-Role Reference

## Intent

Give the user a read-only reference that assigns every current managed-club player to the tactic position and role where the player has the strongest individual fit. The reference ignores squad need, depth strings, existing Planner assignments, and competition from other players.

## Delivered behavior

- The Planner toolbar opens a **Best role fit reference** Modal. It defaults to In Possession, Current, the first tactic lane, and Current descending.
- The user can select In Possession or Out of Possession and choose Current or Potential as the assignment basis. The selected lane shows both adjusted score columns. Sorting Name, Current, or Potential changes only row order.
- Rust assigns each player in the exact current managed-club cohort to one eligible lane, or to **No eligible role** when the selected basis has no eligible score. The reference is scoped to the active save and current snapshot.
- A lane requires selected-phase familiarity of at least 12. Familiarity from 12 through 15 and a preferred-foot mismatch each deduct five points, with a zero floor. A strict foot mismatch is ineligible. Qualified placements use their base position.
- Potential uses the existing CA-to-PA visible-attribute projection. The feature does not change tactics, Planner assignments, scores, snapshots, or persisted data.

## Final architecture

- Planner-private Rust fit helpers provide both single-phase reference scoring and the optimizer's existing linked-lane scoring. The optimizer retains its two-phase behavior.
- `role_reference` resolves the active save, current snapshot, exact managed-club cohort, tactic, current role scores, and potential inputs. It returns tactic-ordered lane groups with deterministic tactic-order ties and a separate no-eligible collection.
- React requests a save-, phase-, and basis-keyed read only while the Modal is open. The Modal selects among Rust-owned lane groups and sorts its bounded rows locally without recomputing fit or assignment.
- No schema, migration, persistence, route, dependency, ADR, or debug report was required.

## Validation and limitations

- `./scripts/dev format` passed.
- The Planner route suite passed 106 tests.
- `./scripts/dev test` passed 543 tests, with the two approved unrelated release-test failures.
- `./scripts/dev check` passed, including 546 Rust tests with 2 ignored.
- `./scripts/dev smoke` passed 48 of 48 tests.
- `git diff --check` passed for the recorded feature range.
- The feature-complete review cleared correction round 1: Blocking **No**, with no CRITICAL, HIGH, MEDIUM, or NITPICK findings.
- Repowise was unavailable. Direct source, test, configuration, and Git evidence were used instead.
- Chromium smoke does not prove a native Tauri/WebView path. Native Tauri/WebView and populated Modal-density checks at the supported desktop sizes remain unverified.

## Exact implementation refs

**Base:** `bd3b47137ac7c2bfbbcc723dc58b6ca5e47d96a7`

There are no earlier PR merge refs. The documentation reconciliation ref is pending the documentation-only checkpoint.

| Ref | Subject | Role |
| --- | --- | --- |
| `7a6135d094b218152904f5b378a16bf4dacb19bb` | `docs(planner): plan best-role reference` | Planning record |
| `f9db455d0b6892b10a93b243e90a2e1fc2b3e0a7` | `refactor(planner): share tactic fit scoring rules` | Shared Planner fit rules and optimizer characterization |
| `34ee620b29f998b6dab70c6b3f579c766575d9eb` | `feat(planner): rank players by best tactic role` | Read-only Rust reference service, command, and coverage |
| `d9d51dc640ba3c9529810c6f258c29bf3c401690` | `feat(planner): show best-role reference` | Query adapter, Planner Modal, route tests, smoke path, and design note |
| `98247ca67629720379c28fd5a501f46fc5b54df4` | `fix(planner): refresh role reference after tactic saves` | Tactic-save invalidation and reference pitch correction |
| `Pending record` | `docs(planner): archive best-role reference feature` | Documentation reconciliation |

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/planner-best-role-reference
base_branch: main
base_ref: bd3b47137ac7c2bfbbcc723dc58b6ca5e47d96a7
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_check_name: check
ci_repair_rounds: 0 of 2
earlier_pr_merge_refs: none
feature_review_blocking: false
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
implementation_range: "bd3b47137ac7c2bfbbcc723dc58b6ca5e47d96a7..98247ca67629720379c28fd5a501f46fc5b54df4"
final_pr_commit_set:
  - 7a6135d094b218152904f5b378a16bf4dacb19bb
  - f9db455d0b6892b10a93b243e90a2e1fc2b3e0a7
  - 34ee620b29f998b6dab70c6b3f579c766575d9eb
  - d9d51dc640ba3c9529810c6f258c29bf3c401690
  - 98247ca67629720379c28fd5a501f46fc5b54df4
  - Pending record (documentation reconciliation)
correction_ref: 98247ca67629720379c28fd5a501f46fc5b54df4
close_out_documentation_ref: Pending record
```

## Feature close-out

**State:** Current. Validation, feature-complete review, and documentation reconciliation cleared the exact implementation set. The final PR is ready for publication but remains unpublished and unmerged.

## Follow-up

- Publish only through the GitHub publication workflow when the branch is ready for review. Resolve the documentation reconciliation ref from Git after the documentation-only checkpoint.
- Run native Tauri/WebView and populated Modal-density checks at the supported desktop sizes when that environment is available.
