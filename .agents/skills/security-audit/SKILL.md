---
name: security-audit
description: Read-only security audit for trust boundaries, secrets, auth, authorization, injection, and AI footguns. Use with $workflow-security-audit, when the user asks if code is safe, wants a vulnerability review, or before first deploy after auth, payments, or sensitive data handling.
---

# Security audit

Read `.agents/skills/technical-writing/SKILL.md` before writing audit prose in chat.

This skill runs a **defensive** audit of code the developer controls. Report findings and fixes in the report. Do not build exploit payloads or run attacks against live systems.

`$workflow-review` and the `reviewer` role cover functional bugs and architecture. They do **not** replace this pass. Calibrate depth to hobbyist solo-dev scope in `AGENTS.md` — report real reachable issues, not exhaustive hardening theater.

## Bundled references

| Path | When to load |
| --- | --- |
| `references/universal.md` | Every audit — read first |
| `references/testing.md` | Vitest setup, Playwright e2e, `mockIPC`, fixtures, CI test artifacts |
| `references/react.md` | React under `src/`, Vite SPA, TanStack Router, Query, or Zustand |
| `references/vite.md` | `vite.config.ts`, Vitest config, `index.html`, `.env*`, build output, Playwright `webServer` |
| `references/rust.md` | Rust under `src-tauri/` — SQL, fs, process, Serde, deps (pair with `tauri.md` for IPC ACL) |
| `references/tauri.md` | Tauri desktop — `src-tauri/`, IPC, capabilities, `tauri.conf.json`, `@tauri-apps/*` |
| `references/csharp.md` | C# under `bridge/` — BepInEx plugin, file protocol, memory interop, diagnostics |
| Other `.agents/skills/` whose description matches security, the stack, auth, payments, database, or mobile | When recon finds matching technology |

Load `react.md` and `vite.md` for this template when `src/` or the Vite toolchain is in scope. Load `rust.md` and `tauri.md` together when `src-tauri/` or IPC is in scope — Rust sinks and capability ACL are one story. Load `csharp.md` when `bridge/` or the FM file protocol is in scope — pair with `rust.md` for the full request → scan → dump path. Load `testing.md` when the scoped diff touches test setup, mocks, or e2e stubs.

Derived stack templates may add `security-*` skills or framework reference files. Load them during recon when the stack matches. Do not load stack refs when recon shows the technology is not in use.

## Process

### 1. Scope the target

Use `the user-supplied scope` when the command supplies a path, feature name, or comparison base. Otherwise:

| Mode | Default scope |
| --- | --- |
| **scoped** | Active feature ledger diff (`git diff <base>...HEAD`) or `git diff --cached` when the developer audits staged work |
| **full** | Repository root — developer must pass `full` or a path in arguments |

Confirm the scope path exists and is non-empty before deep inspection.

### 2. Reconnaissance

Map the attack surface before hunting. Keep recon read-mostly. Determine:

- **Languages and frameworks** — manifests (`package.json`, `pyproject.toml`, `go.mod`, etc.) and directory shape
- **Entry points** — HTTP routes, GraphQL, webhooks, CLI args, queue consumers, file uploads, LLM inputs
- **Dangerous sinks** — raw SQL, shell/exec, templates, file paths, outbound HTTP, redirects, deserializers
- **Trust boundaries** — where auth runs, how authorization is enforced, session or JWT handling
- **Data stores and platforms** — SQL, NoSQL, managed auth or database platforms
- **Config and secrets** — env handling, CORS, security headers, debug flags

Read `references/universal.md`, `references/testing.md` when test infrastructure is in scope, `references/react.md` when the surface map includes React or this template's Vite SPA stack, `references/vite.md` when Vite config, env files, or build artifacts are in scope, `references/rust.md` and `references/tauri.md` when recon finds `src-tauri/`, Tauri config, capabilities, or IPC, `references/csharp.md` when recon finds `bridge/` or BepInEx plugin code, and any matching stack or security skills from `.agents/skills/`.

Produce a short **surface map**: stack, concrete entry points with file paths, platforms in play, and conspicuously unguarded areas. Show it to the developer before the deep pass when the scope is larger than a single file.

### 3. Select checks

Load vulnerability depth only when recon gives a foothold. Examples:

| Recon signal | Check focus |
| --- | --- |
| Raw SQL or string-built queries | Injection, unsafe raw helpers |
| User data in templates or DOM | XSS, SSTI if templates exist |
| `exec`, `eval`, `child_process`, shell calls | Command execution |
| File paths or uploads from input | Path traversal, unsafe uploads |
| Outbound HTTP from user-supplied URL | SSRF |
| Redirect from request param | Open redirect |
| Resource by id without ownership check | IDOR, broken function-level auth |
| Bulk create or update from body | Mass assignment |
| Login, JWT, session, password reset | Authentication flaws |
| Supabase, Firebase, Convex, or similar | Platform rules and RLS |
| Stripe or payment routes | Client-trusted prices, webhook verification |
| LLM prompts from user content | Prompt injection, key exposure |
| Tauri IPC, capabilities, plugins | Command validation, capability scope, CSP, plugin over-permissioning |
| `vite.config.ts`, `.env*`, `dist/` | Env prefix gate, `define` leaks, source maps, dev server bind, `public/` dumps |
| `src-tauri/` Rust without new IPC | SQLi, path traversal, `Command`, Serde, `cargo audit` gap |
| `mockIPC`, Playwright stubs, fixtures | Committed secrets, permissive mocks, test code in prod bundle |
| `bridge/`, BepInEx plugin, file protocol | Read-only memory violations, `request.json` deserialization, path traversal on dump writes, `unsafe`/P/Invoke, diagnostics PII |

Skip classes with no foothold. List skipped classes in the report so the developer sees audit edges.

### 4. Deep pass

**Default (solo-dev):** one read-only pass tracing attacker-controlled input from source to sink in the scoped code.

**Optional parallel pass:** when recon selects **four or more** distinct check clusters, spawn up to three read-only subagents in parallel. Give each cluster a subset of classes, the surface map, and the relevant skill paths. Each specialist reports only — no fixes.

For every issue report:

- **Severity** — Critical, High, Medium, Low (see Output contract)
- **Class** — short label (e.g. IDOR, exposed secret)
- **Location** — `file:line`
- **Flow** — how attacker input reaches the sink when traced
- **Impact** — what an attacker can do in concrete terms
- **Fix** — short before/after or remediation step
- **Confidence** — confirmed (path traced) or suspected (not fully traced)

Do not report generic best-practice items with no reachable sink in scope.

### 5. Aggregate and present

Merge specialist output if used. Deduplicate same line flagged twice. Lead with the single most serious **confirmed** finding in one sentence.

Use read-only inspection commands only. Do not edit, write, stage, unstage, commit, or push. Do not auto-fix — the developer delegates remediation via `$workflow-fix` or manual edits.

## Output contract

```text
# Security audit — <scope>

## Summary
- Surface: <one line from recon>
- Confirmed: <N> Critical, <N> High, <N> Medium, <N> Low
- Suspected: <N> (listed below)
- Classes covered: <list>
- Classes skipped (no foothold): <list>

## Confirmed findings
(ordered by severity, then reachability)

### [SEVERITY] <Class> — <file:line>
- Flow: ...
- Impact: ...
- Evidence: <short snippet>
- Fix: ...

## Suspected / needs verification
(same shape)

## Not assessed
<classes skipped and why>
```

### Severity guide

| Tier | Meaning | Examples |
| --- | --- | --- |
| **Critical** | Unauthenticated or low-effort exploit; full auth bypass; exposed service-role or admin key; open database rules | Public endpoint with no auth on destructive action; `service_role` in client bundle |
| **High** | Authenticated exploit with clear impact; client-trusted payment price; IDOR on sensitive records | Any user can read another user's data by id |
| **Medium** | Exploit needs unlikely conditions or limited blast radius | Missing rate limit on non-critical endpoint |
| **Low** | Defense-in-depth; hard to reach; minor information leak | Verbose error on edge path only |

**Critical** and **High** confirmed findings should block release or deploy until fixed or explicitly approved by the developer. **Suspected** findings do not block by default.

## When generating code

Consult `references/universal.md`, `references/react.md` when touching `src/`, `references/vite.md` when touching Vite config or client env, `references/rust.md` and `references/tauri.md` when touching `src-tauri/` or Tauri config, `references/csharp.md` when touching `bridge/` or the FM file protocol, `references/testing.md` when adding mocks or fixtures, and matching stack skills before writing auth, payments, database access, API keys, or user data handling. Prevention is cheaper than audit.

## Recallium

Read `.agents/skills/recallium-usage/SKILL.md` and `AGENTS.md`. Use `project_name` from `AGENTS.md` § Recallium project.

**Search before:** audit — prior security decisions or incidents not in wiki or Git.
**Save:** do not save by default — report only. Save one concise memory only when the developer asks to record a durable security decision not captured in wiki or ADR.
