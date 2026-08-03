# Planned Feature Specs

> **Authority:** These files own pre-implementation feature intent and behavioral detail for sequencing (`$workflow-roadmap`) and later planning (`$workflow-plan-feature`). They do not own delivery order — that lives in [TODO.md](../../TODO.md) after `$workflow-roadmap` approval. They do not own PR and commit delivery plans — those live in [active ledgers](../active/README.md) once work begins.

## When to create a spec

Create a spec when you can describe user-visible behavior, data touched, and integrations — even if details are still provisional.

**Bullet-only MVP items in CONCEPT.md are enough to use `$workflow-roadmap`** — the skill still produces a dependency-aware sequence, with lower confidence until planned specs ground the dependencies. Add specs later to firm up ordering before `$workflow-plan-feature`.

## Filename

`<feature-slug>.md` — lowercase, hyphens, matches the slug used in TODO and future active ledgers.

## Spec template

~~~markdown
# <Feature Name>

## Summary

One paragraph: what capability this adds and why it matters for MVP.

## User-visible behavior

- ...
- ...

## Data and persistence

- Entities or fields introduced or changed
- Migration or schema expectations
- Reads vs writes

## Dependencies on other features

- **Requires:** features or foundations that must exist first
- **Enables:** features that become easier or possible after this

## Integrations and boundaries

- External services, APIs, auth surfaces, shared UI patterns

## Non-goals (this feature)

- ...

## Open questions

- ...

## Acceptance sketch

Concrete scenarios that prove the feature works — not full test cases.
~~~

When `$workflow-plan-feature` promotes the feature, copy all accepted detail into the matching active ledger sections, update links to target the ledger, and delete the planned spec in the same planning change. The active ledger becomes the sole owner of feature intent and delivery state.
