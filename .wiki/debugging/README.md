# Debug Reports

This directory contains short, durable reports for confirmed failure patterns that code, regression tests, and Git history do not explain well enough on their own.

## When to write a report

Write a report only when the root cause is confirmed and at least one of these conditions applies:

- The failure pattern could recur in another module or project.
- The cause depends on an environment, toolchain, external system, timing condition, or hidden state.
- Diagnosis or recovery requires non-obvious steps.
- The failure caused or credibly risked data loss, corruption, a security incident, or an outage.
- A future agent would otherwise repeat a substantial investigation.

An ordinary bug with a clear regression test and commit does not need a report. Keep raw logs, failed hypotheses, temporary instrumentation, and experiment output in `.work/`; remove them when the work is complete.

Use a dated lowercase filename such as `2026-08-03-stale-cache-after-replacement.md`. Never include credentials, tokens, personal data, full production dumps, or unsanitized logs.

## Report format

```markdown
# <Failure Pattern>

## Status

Resolved | Recurring | Superseded

## Scope

- First confirmed: YYYY-MM-DD
- Affected components:
- Affected environment or versions:

## Symptom

What was observed? Include only short, sanitized error text when it helps identification.

## Confirmed root cause

What caused the failure? State the evidence that ruled out plausible alternatives.

## Resolution

What changed, and where?

## Prevention and regression coverage

- Tests:
- Validation or guard:
- Commit or feature record:

## Future diagnosis

What is the shortest reliable way to recognize or investigate this pattern again?

## Related records

- ADR:
- Feature record:
- Supersedes:
```

Update a report when its diagnostic procedure or scope changes. Mark it `Superseded` and link its replacement when the original explanation no longer applies.
