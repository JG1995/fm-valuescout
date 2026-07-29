---
name: coding-standards
description: Coding standards for implementation and review. Read before writing or reviewing production code and tests. Load references/universal.md always; load stack and project references when ARCHITECTURE or touched files match.
---

# Coding standards

Read `.cursor/skills/technical-writing/SKILL.md` before writing prose in audit reports, wiki updates, or comments meant for humans.

This skill defines **how code should look and behave**. `.wiki/ARCHITECTURE.md` defines **where code lives and how the system is shaped**. They complement each other — read both when structure and style both matter.

## Bundled references

| Path | When to load |
| --- | --- |
| `references/universal.md` | Every implementation and code review — read first |
| `references/testing.md` | Every `/build`, `/build-loop`, `/fix`, `/checkpoint`, and code review that touches or adds tests |
| `references/react.md` | App code under `src/` — Bulletproof React layout, TanStack Router, Query, Zustand, components |
| `references/vite.md` | `vite.config.ts`, Vitest, TS config, env vars, `package.json` scripts |
| `references/rust.md` | Rust code under `src-tauri/` — module layout, errors, clippy, tests |
| `references/tauri.md` | Tauri config, capabilities, IPC commands, plugins (when present) |
| `references/csharp.md` | C# under `bridge/` — BepInEx IL2CPP plugin, file protocol, memory scan |
| `references/<stack>.md` | Other stacks when `.wiki/ARCHITECTURE.md` names them (e.g. `nextjs.md`) |
| `references/<project>.md` | When present — project-specific conventions not covered elsewhere |

Load `react.md` and `vite.md` for this template when `src/` exists or the diff touches the application toolchain. Load `rust.md` when `src-tauri/` exists or the diff touches the Rust crate; load `tauri.md` when IPC, capabilities, or Tauri config are in scope. Load `csharp.md` when `bridge/` exists or the diff touches the BepInEx FM26 plugin.

**Cross-boundary decisions** (where computation runs, SQL execution, file I/O, validation at trust boundaries, IPC payload size): load **`react.md` + `tauri.md` + `rust.md`** together — not only the file that matches the diff path. The thin-thick split and database rules span all three.

**FM bridge decisions** (memory read, file protocol, layout pins, dump output): load **`csharp.md` + `rust.md`** together — C# owns in-process scan; Rust owns orchestration and ingest validation.

Do not load stack refs when recon shows the technology is not in use.

Other skills in `.cursor/skills/` (e.g. `ui-design`) may add domain rules. When their description matches the work, read them alongside this skill.

## When to apply

| Phase | Requirement |
| --- | --- |
| `/build`, `/build-loop`, `/fix` | Read this skill + `universal.md` + `testing.md` when the commit adds or changes tests + matching refs before editing production code or tests |
| `/checkpoint`, `/review`, `reviewer` | Read before judging conventions in the diff |
| `/plan-feature` | Optional for delivery planning; required when the plan names concrete patterns |

## Before you add code

1. Read `references/universal.md`.
2. Read matching stack and project references from the table above.
3. Run the **intentional minimalism** decision ladder in `.cursor/skills/minimalism/SKILL.md` before adding dependencies, files, classes, or abstractions. Minimalism governs scope; universal governs shape.

State which ladder rung you stopped at in `/build` when the decision matters for the commit.

## During implementation

- Match existing style in the files you touch.
- Apply universal rules unless a stack or project reference overrides them.
- Validate at trust boundaries per `AGENTS.md` safety carve-outs.
- Do not drive-by refactor unrelated code.

## During review

Flag violations of loaded references as **HIGH** when they break project conventions or layer boundaries, or **MEDIUM** when they harm readability without functional impact. Functional bugs remain the reviewer's first priority per `.cursor/agents/reviewer.md`.

## Adding references in a derived project

1. Add `references/<stack>.md` or `references/project.md` under this skill directory.
2. Document stack-specific sinks, file placement, naming, and patterns that override universal rules.
3. Name files after the stack or concern (`nextjs.md`, `postgres.md`, `project.md`).
4. Keep each reference focused. Split when a file grows past one concern.

Do not duplicate `ARCHITECTURE.md` — link layer boundaries there; put line-level and module-level rules in references.
