# Universal coding standards

Load this file on every implementation and code review pass. Stack- and project-specific references in `references/` override rules here when they conflict. See **Precedence** below.

Calibrate depth to hobbyist solo-dev scope in `AGENTS.md`. Structural quality stays high; skip paranoid padding on paths that cannot fail in practice.

## Precedence

When a stack or project reference (`references/nextjs.md`, `references/rust.md`, `references/project.md`, or a matching skill under `.cursor/skills/`) defines an idiomatic pattern that conflicts with a rule below, **the stack or project rule wins**.

Examples:

- **Error handling** — `Result<T, E>` and `?` in Rust, or typed error unions in TypeScript, replace generic "use exceptions" guidance where the stack reference says so.
- **Nullability** — `Option`, `string | null`, or discriminated unions are idiomatic; bare nullable fields without a type-level signal remain discouraged when a better type exists.
- **Argument counts** — framework composition patterns (e.g. React component props) may relax strict argument limits when the stack reference documents the exception.

## Meaningful names

- Names reveal intent — why something exists, not only what it holds.
- Avoid disinformation and empty labels (`data`, `info`, `manager` with no specific meaning).
- Use pronounceable, searchable names.
- Types and classes: nouns (`UserAccount`, `PaymentProcessor`).
- Functions and methods: verbs (`calculateTotal`, `sendEmail`).
- Avoid mental mapping, encodings, and Hungarian notation.

## Functions

- Keep functions small. Under 20 lines is a good target when clarity allows.
- One responsibility per function — one level of abstraction inside the function body.
- Prefer 0–2 arguments; three is a soft cap. Avoid flag arguments that change behaviour (`render(true)`).
- No hidden side effects — the function should do what its name says.
- Separate commands (change state) from queries (return information).

## Comments

- Prefer self-explanatory code over comments.
- Good comments: legal requirements, non-obvious warnings, public API documentation.
- Bad comments: restate the code, explain bad code instead of fixing it, or comment out dead code — delete dead code; Git keeps history.
- If a comment is needed, consider refactoring first.
- Ponytail comments (`# ponytail:` in `AGENTS.md`) mark deliberate simplifications with an upgrade trigger — not TODOs.

Prose in wiki, prompts, and user-facing docs: follow `.cursor/skills/technical-writing/SKILL.md`.

## Formatting

- Keep files focused. Split when a file mixes unrelated concerns.
- Group related code vertically. Use blank lines between concepts.
- Limit line length (80–120 characters when the formatter allows).
- Use consistent indentation and match existing project style.

## Objects and data

- Objects hide data and expose behaviour where that models the domain.
- Data structures expose data with minimal behaviour where that models the domain.
- Law of Demeter: talk to immediate collaborators; avoid `a.getB().getC().doSomething()` chains.
- Do not expose internal structure through getters and setters without a reason.

## Error handling

- Prefer typed, explicit error paths over magic return codes or silent failure.
- Provide context in error messages at trust boundaries.
- Avoid returning `null` when an empty collection or optional type is clearer.
- Avoid passing `null` as an argument when a typed optional exists.
- At trust boundaries (input validation, persistence, security): invest full rigour per `AGENTS.md` safety carve-outs.
- Inside trusted code paths: one clear validation at the boundary is usually enough for solo-dev scope.

## Classes and modules

- Small units measured by responsibilities, not only line count.
- Single reason to change per module or class when practical.
- High cohesion: members relate to the same purpose.
- Low coupling: limit dependencies across boundaries defined in `ARCHITECTURE.md`.
- Open for extension, closed for modification — without speculative abstraction.

## Unit tests

See `references/testing.md` for TDD scope, the test quality gate, and dumb-test patterns.

At a glance: follow F.I.R.S.T., one concept per test, test names describe behaviour, Arrange–Act–Assert structure. Cover critical paths and data-loss risks. Skip trivial one-liners per `AGENTS.md`.

## Code quality principles

- **DRY** — do not duplicate logic; duplicate is not duplication when cases may diverge.
- **YAGNI** — do not build for hypothetical futures.
- **KISS** — prefer the simplest design that meets the current contract.
- **Boy Scout Rule** — leave touched code slightly cleaner when the change is already in scope.

## Intentional minimalism (decision ladder)

Before you add a dependency, function, class, or file, run the ladder in `.cursor/skills/minimalism/SKILL.md`. Stop at the first rung that solves the problem:

1. **YAGNI** — does this need to exist?
2. **Standard library** — does the runtime already provide it?
3. **Native platform** — does the browser, OS, or platform provide it?
4. **Existing dependency** — does an installed package already cover it?
5. **One line** — can one clear line solve it?
6. **Minimum code that works** — shortest implementation that passes tests; no extension points until needed.

Minimalism governs **how much** code to write. This file governs **how that code should look and behave**.

## Code smells to avoid

- Long functions or classes
- Duplicate logic (not duplicate structure that may diverge)
- Dead code — unused variables, functions, parameters
- Feature envy — a method more interested in another module's data than its own
- Inappropriate intimacy — modules knowing too much about each other's internals
- Long parameter lists
- Primitive obsession — overusing primitives when a small type clarifies intent
- Switch or match sprawl when a simpler map or polymorphism fits the stack

## Concurrency

When the code uses threads, async tasks, or shared mutable state:

- Keep concurrent code easy to find and reason about.
- Limit the scope of locked or shared data.
- Use thread-safe or async-safe primitives from the stack.
- Know the execution model (event loop, worker pool, etc.) before changing concurrent code.

## System design

- Separate construction from use where the stack supports it (factories, builders, dependency injection).
- Program to interfaces or traits when they clarify boundaries — not for every call site.
- Favor composition over inheritance when both fit.
- Apply patterns when they simplify; skip patterns that only add ceremony.

## Refactoring

- Refactor in small steps while tests stay green.
- Have passing tests before and after each step.
- Common moves: extract function, rename, move, inline — only within the current commit or fix scope.

## Documentation

- Self-documenting code first, then comments, then external docs.
- Public APIs and command surfaces need clear documentation when the change introduces them.
- Keep durable project truth in the wiki per `.wiki/INDEX.md` ownership rules.

## Core philosophy

Code is read more often than it is written. Optimize for readability and maintainability, not cleverness.
