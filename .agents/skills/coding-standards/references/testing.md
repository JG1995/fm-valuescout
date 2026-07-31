# Testing and TDD

Load this file on every `$workflow-build`, `$workflow-build-loop`, `$workflow-fix`, `$workflow-checkpoint`, and code review pass that touches or adds tests.

Security risks in mocks, fixtures, and CI test artifacts live in `security-audit/references/testing.md` — complementary to this file (TDD quality vs test-infra exposure).

TDD is the default for **behavioural** work — not a ritual for every line of code. A test must earn its place by protecting something that matters.

## When to use RED → GREEN → REFACTOR

Use test-first when the commit changes behaviour, contracts, or data paths:

- New user-visible behaviour or API contract
- Bug fix — reproduce with a failing test first (`debug` skill)
- Data loss, persistence, migration, or auth paths
- Regression risk — logic that broke before or is easy to break again
- Integration between modules the ledger names as in scope

## When TDD is optional or skip tests

Skip new tests when the change is genuinely trivial and risk-free per `AGENTS.md`:

- Documentation, comments, or wiki-only edits
- Mechanical rename or move with no behaviour change (existing tests cover behaviour)
- Config template or scaffolding with no runtime behaviour yet
- Trivial one-liners (`get_timestamp()`, pure constant export) with no branch logic

When you skip tests, say why in the checkpoint package — one line is enough.

## Test quality gate

Before you treat a test as the RED step or mark a commit test-complete, pass this gate. If any answer is weak, rewrite the test or skip testing with justification.

### 1. What failure does this test prevent?

State the **expected failure mechanism** in one sentence:

- Wrong output on a realistic input
- Missing error when input is invalid at a trust boundary
- Regression of a specific bug or contract

**Fail the gate** when the test only checks that code runs, that a mock was called, or that `true` is true.

### 2. Would this test fail if the behaviour were wrong?

Imagine the implementation returns wrong data, skips validation, or uses the wrong branch. The test must **fail** for that wrong implementation.

**Fail the gate** when the test would still pass if you deleted the real logic and returned a hardcoded value.

### 3. Does the test exercise real behaviour?

Prefer real collaborators within the commit scope. Use mocks only to isolate **slow**, **external**, or **non-deterministic** boundaries — not to avoid setting up the behaviour under test.

**Fail the gate** when mocks stand in for the module you are actually changing.

### 4. One concept per test

One behaviour or edge case per test. Name describes the scenario and expected outcome.

Good: `rejects checkout when cart total is negative`

Bad: `testUserService` (no behaviour stated)

### 5. RED must fail for the right reason

Run the test before implementation. Failure must be **missing or wrong behaviour**, not syntax error, import error, or misconfigured test harness.

Record RED evidence in checkpoint: what failed and why that failure matches the gate.

## Dumb tests to reject

| Pattern | Why it fails the gate |
| --- | --- |
| Assert not null / assert defined only | Passes for empty or wrong objects |
| Test that framework or runner works | Proves tooling, not product behaviour |
| Snapshot of entire component output without selective assertions | Breaks on any change; often ignores wrong logic |
| Mock returns fixture; test asserts mock called | Proves wiring, not outcome |
| Duplicate production logic in the test | Test cannot detect wrong implementation |
| Coverage-driven test with no stated risk | Ceremony without protection |
| Testing private methods directly | Brittle; test behaviour through public contract |

## Meaningful test checklist

A test that passes the gate usually has:

- **Arrange** — realistic inputs or state (not only empty defaults)
- **Act** — the operation the user or caller performs
- **Assert** — observable outcome (return value, state change, thrown error, side effect at boundary)

For integration tests: assert on **outcome** at the system edge (HTTP response, DB row, message published), not internal call order unless order is the contract.

## Calibrate to solo-dev scope

- Cover critical paths and data-loss risks. Do not test every branch for ceremony.
- One good behavioural test beats five shallow tests.
- Stack-specific testing patterns (component testing, property tests, E2E scope) live in `references/<stack>.md` when added.

### Playwright smoke scope

`./scripts/dev smoke` is **not** full-stack proof. Playwright drives Chromium against Vite with a stub IPC layer — see [.wiki/ARCHITECTURE.md](../../../../.wiki/ARCHITECTURE.md) §6.4 Playwright smoke scope for the covered / not-covered table and layer ownership.

Stack stub pattern and ponytail upgrade trigger: `references/tauri.md` § Playwright smoke — browser without WebView.

## Reviewer use

Flag as **HIGH** when:

- Stated commit outcome has no test and the gate above says tests were required
- RED step missing or RED failed for setup/syntax instead of behaviour
- New test fails the quality gate (would pass with wrong implementation, mock-only, or no stated failure mechanism)

Flag as **MEDIUM** when a secondary path the commit claims to cover has a shallow test.

Do not flag **NITPICK** for "add more tests" on paths the commit and ledger explicitly defer.
