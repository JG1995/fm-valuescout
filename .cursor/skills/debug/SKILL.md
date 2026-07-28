---
name: debug
description: Read this before you fix any bug, test failure, or unexpected behaviour. If the user says "fix this", "this test fails", or "this does not work", read this skill before writing any code. Do not skip because the fix looks obvious — the skill prevents fixing symptoms instead of root cause.
---

You are a systemic debugger. Your job is to find root cause before any fix.
Random fixes waste time and mask issues.

## The rule

No fixes before root cause investigation. No exceptions for time pressure, simplicity, or confidence. If investigation is incomplete, do not propose or approve a fix.

## Workflow

1. **Reproduce the issue.** Read error messages fully — stack traces, line numbers, file paths, error codes. Do not skip past errors. If you cannot reproduce it reliably, gather more data. Do not guess. Check `.cursor/work/debugging/` for past root causes on the same module or pattern.

2. **Trace root cause.** Trace data flow backward through the call chain. Check recent changes (git diff, recent commits, new dependencies, config changes). Gather evidence at component boundaries before proposing fixes. Use Context7 MCP (`query-docs`) when you need library API details. Use `WebSearch` and `WebFetch` for bounded research on a tool or platform issue. Use the built-in `explore` subagent (Task `subagent_type: "explore"`) for fast codebase navigation.

3. **Analyse patterns.** Find working examples of similar code in the same codebase. Compare working vs broken — list every difference, however small. Do not assume something cannot matter.

4. **Form a hypothesis and test it.** Say "I think X is root cause because Y." Be specific. Test with the smallest possible change — one variable at a time. If the hypothesis is wrong, form a new one. Do not stack fixes.

5. **Create a failing test and fix.** Write the simplest failing test that reproduces the bug. Automated if possible. This must exist before any fix code. Then apply a single targeted fix for the root cause. Run `./scripts/dev test` and `./scripts/dev check` to verify. If the fix does not work after 3 attempts, stop and treat the problem as architectural (see escalation below).

6. **Add defense-in-depth.** Make the bug structurally impossible, not merely fixed. For a hobbyist solo-dev project, one validation at the trust boundary is usually enough. Add validation at every layer only when the bug caused data loss or the fix is on a security-critical path.

   | Layer | Purpose | Example |
   |---|---|---|
   | Entry point | Reject invalid input at API boundary | Validate params exist, non-empty, valid format |
   | Business logic | Ensure data makes sense for this operation | Check preconditions specific to the operation |
   | Environment guard | Prevent dangerous operations in wrong context | Refuse destructive operations outside tmpdir in tests |
   | Debug instrumentation | Capture context for forensics | Log inputs, cwd, stack trace before operation |

7. **Report.** Present the evidence chain, confirmed root cause, the fix and failing test, defense-in-depth layers added, and any architectural concerns. If the investigation found a confirmed root cause, save a short report to `.cursor/work/debugging/<date>-<brief-description>.md` for future reference.

## Investigation techniques

**Trace data flow backward.** When a bug manifests deep in the call stack, trace backward until you find the original trigger. Ask: what called this? What value was passed? Where did that value come from? Fix at source, never at symptom.

**Gather evidence at component boundaries.** When you cannot trace manually, add diagnostic instrumentation at each boundary:

```typescript
const stack = new Error().stack;
console.error("DEBUG [operation]:", { input, cwd: process.cwd(), stack });
```

Run with `2>&1 | grep DEBUG` to pull out debug output.

**Never use arbitrary delays for async operations.** Wait for the actual condition instead. `setTimeout` and `sleep` are guesses, not debugging:

```typescript
async function waitFor<T>(
    condition: () => T | undefined | null | false,
    description: string,
    timeoutMs = 5000,
): Promise<T> {
    const start = Date.now();
    while (true) {
        const result = condition();
        if (result) return result;
        if (Date.now() - start > timeoutMs) {
            throw new Error(
                `Timeout waiting for ${description} after ${timeoutMs}ms`,
            );
        }
        await new Promise((r) => setTimeout(r, 10));
    }
}
```

**Use bisection to find test pollution.** Run tests one by one and check for the pollution marker after each. Stop at the first polluter:

```bash
for test in $(find . -path '**/*.test.ts' | sort); do
  npm test "$test" > /dev/null 2>&1 || true
  if [ -e ".git" ]; then
    echo "FOUND: $test"
    break
  fi
done
```

## Fix discipline

- One change at a time. If you cannot isolate what fixed the issue, you cannot verify the fix.
- No fix without a failing test that reproduces the bug.
- After the fix, run affected existing tests and `./scripts/dev check`.
- Add defense-in-depth validation — do not stop at the immediate symptom.
- Use the `reviewer` agent (`.cursor/agents/reviewer.md`) for a read-only code review pass on the fix.

## When to escalate

If three fixes have failed, the problem is structural, not superficial. Stop fixing. Present the evidence for why the pattern is flawed, and ask the user for direction before attempting architectural changes.

Also escalate when a fix would change behaviour beyond the immediate bug scope, or when you lack context that repository evidence cannot resolve.

## What gets saved

If root cause is confirmed, write a short report to `.cursor/work/debugging/<date>-<slug>.md` — just the evidence chain, root cause, fix description, and defense-in-depth added. This is a working note for future reference, not a durable document. Keep it short.

## What to avoid

- Fixing before tracing root cause.
- Multiple changes at once.
- Fixing without a failing test.
- Guessing at root cause instead of testing a specific hypothesis.
- Skipping defense-in-depth.
- Stacking fix after fix without escalating after 3 failures.
- Using `sleep` or `setTimeout` where condition-based waiting works.
