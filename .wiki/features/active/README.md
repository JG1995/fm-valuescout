# Active Feature Ledgers

Keep one ledger per feature in active development. The ledger owns feature intent, the delivery plan, PR boundaries, commit packets, validation evidence, and discoveries that change the plan. It does not own permanent current-state architecture.

Create a ledger with `/skill:workflow-plan-feature`. When a planned spec exists, absorb its accepted intent, behavior, boundaries, dependencies, non-goals, open questions, and acceptance detail into the ledger, then delete the spec in the same planning change. New ledgers use schema 2 and receive an independent plan review before acceptance. Keep one PR active and exactly one commit marked `Active` inside it. The first commit records the reviewed planning artifacts on the first fingerprint-authorized branch before implementation.

## Status vocabulary

**Feature:** `Shaping` | `Active` | `Blocked` | `Validation` | `Documentation reconciliation` | `Ready for final publication` | `Abandoned`

**PR:** `Pending` | `Awaiting prior PR merge` | `Active` | `Ready for publication` | `Merged` | `Removed — <reason>`

**Commit:** `Pending` | `Active` | `Blocked` | `Completed` | `Removed — <reason>`

The content commit can mark its own plan item `Completed`, but it cannot contain its own hash. Record the hash in the next normal ledger-bearing commit or during feature reconciliation. Do not create a ledger-only commit only to record a hash.

Do not record execution model profiles in an active ledger. The globally installed PI_SETUP role definitions provide the model and thinking values used for subagent dispatch. When the final implementation commit becomes `Completed`, set the feature status to `Validation`; this does not run feature close-out.

An `Active` PR records its branch and base, but ledger state alone does not grant branch authority. Explicit `/skill:workflow-deliver-feature` invocation may create or switch to every exact branch covered by its valid Delivery fingerprint. A direct build recovery run still needs a separate branch handoff. For a dependent PR, record each predecessor's immutable 40- or 64-character Git object ID and confirm that the synchronized base contains it before activation.

Schema 2 requires one durable **Completed work** row for each completed commit. Record the implementation outcome, validation evidence, test-portfolio result, final review result, and fix-round count before the content commit. Keep `Pending record` as the Git ref until the next normal ledger-bearing commit can resolve it.

At feature completion, reconcile documentation and move the complete ledger to [completed features](../completed/README.md). Preserve every Delivery fingerprint input through final publication and release.

## Commit sizing guidance

Break feature work into as many atomic commits as needed. Aim for at most **200 changed lines of non-test implementation code per commit**. Count handwritten production source, behavior-bearing configuration, and migrations; exclude tests, test fixtures, test infrastructure, documentation, generated artifacts, lockfiles, and mechanical ledger updates.

This is a soft planning target, not a hard limit or delivery gate. Genuine atomicity, correctness, and trunk safety take precedence. If one coherent, revertible outcome must exceed the target, keep it together and explain why in **Size assessment**. Otherwise prefer additional small commits within the same PR. There is no upper limit to the number of commits in a feature ledger. Tests remain outside the size count, but every test still needs contract-level value; never use a test-to-production line ratio as a gate.

## Ledger template

The template shows the required structure. In an actual ledger, replace every placeholder and repeat the complete PR authority fields and implementation packet for every planned PR and commit. Do not leave `...` in an accepted plan.

~~~markdown
# <Feature Name>

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** <SHA-256 from `delivery_state.py` after plan review>

## Release

**Release intent:** none | patch | minor | major

**Release target:** none | <exact SemVer without a `v` prefix>

**Release command:** none | `<exact project release command>`

**Release verification:** none | `<exact command that verifies the release and final merge ref>`

One ledger has one release outcome. The delivery workflow always runs this phase. For `none`, set the other three fields to `none`. For another intent, instantiate the exact automatic-publication wait and verified-release commands from the [early-alpha release runbook](../../notes/early-alpha-release-runbook.md). Do not substitute a manual tag or release.

## Intent

Why the feature exists and what capability it introduces.

## User-visible behavior

- ...

## Invariants

- ...

## Non-goals

- ...

## Current-state map

- Relevant components:
- Data model:
- Persistence and migrations:
- Existing behavioral assumptions:
- Architectural seams:
- Project validation commands:
- Primary risks:

## Feature architecture

Responsibilities and boundaries for this feature.

## Uncertainty register

### Known

- ...

### Assumptions

- ...

### Decisions

- ...

### Unknowns

- ...

### Risks

- ...

## Walking skeleton

The thinnest path through this feature.

## Delivery plan

### PR 1 — <title>

**Status:** Active

**PR ref:** Not published | <GitHub PR URL>

**Merge ref:** Not merged | <immutable merge commit or equivalent>

**Branch:** <short-lived branch>

**Base branch:** <trunk branch>

**Publication provider:** GitHub | <unsupported provider>

**PR template:** <repository-relative path>

**Merge method:** merge | squash | rebase

**Required checks:** GitHub required checks | <exact repository rule>

**Feature close-out:** Not required | Not run

**CI repair rounds:** 0

**Provisional PR title:** `type(scope): imperative description`

**Purpose:** What this PR delivers and why it is a review and merge boundary.

**Depends on:** Prior PRs, features, or foundations.

#### Commit 1 — Record the approved feature plan

**Status:** Active

**Provisional commit:** `docs(<feature>): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/<feature>.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state.
- `.wiki/BACKLOG.md` — accepted deferred-scope changes, or no change.
- `.wiki/features/planned/<feature>.md` — remove after its accepted content is preserved, when present.
- `.wiki/decisions/<decision>.md` — warranted accepted decision only, when present.

**Behavior and data flow:**

- Move planning truth from the planned source into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch and base without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths.
3. Run the ledger classifier and any repository documentation check.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO/BACKLOG ownership rules, and relevant accepted ADR format.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 <skill-directory>/../../scripts/ledger_state.py <active-ledger>` plus the repository documentation check when one exists.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — <first implementation title>

**Status:** Pending

**Provisional commit:** `type(scope): imperative description`

**Work:** One coherent, revertible implementation outcome.

**Size assessment:** Estimated changed non-test implementation lines and either `Within the soft target` or the atomicity reason for exceeding it.

**Out of scope:**

- Work assigned to later commits or PRs, speculative cleanup, and unrelated changes.

**Implementation packet:**

- Give the worker a complete handoff. Resolve repository-known design choices here instead of leaving them implicit or requiring the worker to reconstruct feature architecture.

**Files and responsibilities:**

- `<path or symbol>` — exact ownership, change, and reason.
- Name tests, fixtures, mocks, snapshots, helpers, configuration, migrations, and documentation that this commit adds, modifies, deletes, or deliberately retains.

**Behavior and data flow:**

- Trace the entry point, state or data transformation, boundary calls, persistence or side effects, and observable result.
- Describe the success, empty, error, stale, replacement, rollback, recovery, or partial-failure paths that apply.

**Ordered implementation steps:**

1. Add the smallest RED proof or establish the exact contract-removal proof.
2. Make the minimum coherent production change that turns the proof GREEN.
3. Remove obsolete implementation and test assets owned by an intentional deletion.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Name the exact tests or runtime probes, expected RED failure, GREEN assertion, and important negative or boundary coverage.
- Account for tests, fixtures, mocks, snapshots, and helpers to add, modify, delete, or deliberately retain. Tie each addition or retention to a supported contract and plausible regression.
- For intentional removal, name obsolete protection to remove or rewrite and require an absence test only when reintroduction is plausible and observable.

**Patterns to verify:**

- Name the closest current repository analogues and the choices to copy or deliberately diverge from.

**Constraints and non-goals:**

- State governing invariants, architecture and trust boundaries, compatibility limits, and explicit exclusions.

**Dependencies and sequencing:**

- Name earlier commits, merged PR refs, generated artifacts, migrations, or external prerequisites required before this work.

**Validation:** Exact project commands and expected evidence, from focused proof through the project commit gate.

**Stop conditions:** Concrete facts that require escalation, replanning, or developer input before implementation continues.

**Review mandate:** Up to eight concrete concerns derived from this commit's risks, invariants, data flow, trust boundaries, test portfolio, and likely regression paths.

#### Commit 3 — <next implementation title>

Repeat the complete implementation packet from Commit 2. Do not replace it with a summary or ellipsis in an actual ledger.

### PR 2 — <title>

**Status:** Awaiting prior PR merge

**PR ref:** Not published | <GitHub PR URL>

**Merge ref:** Not merged | <immutable merge commit or equivalent>

**Branch:** <short-lived branch>

**Base branch:** <trunk branch>

**Publication provider:** GitHub | <unsupported provider>

**PR template:** <repository-relative path>

**Merge method:** merge | squash | rebase

**Required checks:** GitHub required checks | <exact repository rule>

**Feature close-out:** Not required | Not run

**CI repair rounds:** 0

**Provisional PR title:** `type(scope): imperative description`

**Purpose:** Why this work needs a separate review and merge boundary after PR 1.

**Depends on:** PR 1

#### Commit 1 — <implementation title>

Repeat the complete implementation packet from PR 1 Commit 2. In an actual ledger, include every required field rather than referring back to this template.

## Active work

**PR:** <number or title>

**Commit:** Record the approved feature plan | <implementation title>

### RED or removal proof

For the planning-artifact commit, state `Not applicable — independently reviewed planning documents only` and name the ledger classifier and documentation check. For an implementation commit, state the smallest failing test or reproducible proof and the plausible wrong behavior it detects. For an intentional removal, name the retired contract, obsolete protection to delete or rewrite, surviving behavior to prove, and why any absence test has value. When automation is not practical, name the focused command or runtime probe and explain the limitation.

### Expected outcome

Observable repository state when this commit is complete.

### Explicit exclusions

What this commit must not include.

## Discoveries and replanning

Record material deviations, blockers, and decisions that change remaining work. State what was planned, what changed, and why.

- ...

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| ... | ... | Pending record | ... | <commands and result summary> | Pass \| Not applicable \| Accepted gap — <developer-approved reason> | Clear \| Accepted findings — <developer-approved reason> | 0 | None |

Schema 2 requires exactly one row for every `Completed` commit. Use the exact full PR and commit headings in the first two cells; abbreviations are invalid. Escape a literal table pipe as `\|`. Use a fix-round count from `0` through `3`. Resolve `Pending record` from Git in the next normal ledger update or during feature reconciliation.

Automated publication currently supports GitHub PRs and GitHub Actions. Record another provider accurately, but expect delivery to stop rather than improvise an adapter. Run `delivery_state.py` after plan review, record its exact Delivery fingerprint, and rerun it before acceptance. The fingerprint covers every PR authority field, every commit packet fingerprint, and the Release block. Any later authority change ends delivery without model judgment. Record an intermediate PR's immutable merge ref when activating its dependent PR. A completed final-feature record may retain its PR URL as the durable publication reference because its own merge ref cannot appear in the content being merged; do not create a metadata-only follow-up commit solely to record that self-referential ref.

## Final validation

List the exact project commands and manual evidence required before feature review.

## Documentation impact

Complete during reconciliation.

## Abandonment record

Include this section only when the developer explicitly abandons the feature. Set the feature status to `Abandoned`, mark every unfinished PR and commit `Removed — <reason>`, remove **Active work**, and resolve every completed evidence Git ref to its immutable 40- or 64-character object ID. List each resolved ref under **Retained refs**, then record:

**Developer approval:** <where the developer approved abandonment>

**Decision:** <why delivery stopped>

**Retained refs:** <completed implementation refs | None — no completed work>

**Cleanup:** <temporary-artifact, open-PR, branch, and unpublished-work disposition>

**Planning disposition:** <TODO and BACKLOG update>

**Resume rule:** Fresh plan required

Set release intent, target, command, and verification to `none`, then recompute and record the Delivery fingerprint for that developer-approved authority change. Move the complete ledger under `features/completed/` as an abandoned outcome. Do not publish it. Later work starts from a fresh plan rather than reactivating this ledger.
~~~
