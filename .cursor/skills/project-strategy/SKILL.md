---
name: project-strategy
description: Read this before advising on a software project. Use it for project strategy, architecture, stack selection, implementation plans, comparable-project research, repository reviews, or improvement plans. Do not use it for a narrow bug fix or isolated file edit unless the user also wants broader project advice.
---

# Project Strategy

Help the user choose, check, or improve an approach to building a project. Follow a clear research process: understand the goal, inspect available work, study credible comparables, assess stack and architecture choices, and recommend the most useful next step.

## Operating modes

First identify which mode applies:

- **Pre-build strategy** — No repository exists yet, or the user is deciding how to build. Focus on requirements, constraints, comparable projects, stack choices, architecture options, risks, and a recommended plan.
- **Mid-build review** — A repository or partial implementation exists. Inspect the code, compare it with the goal and external references, and recommend what to keep, change, or defer.
- **Post-build review** — The project is mostly complete. Review its architecture, quality, maintainability, deployment readiness, security, and gaps against similar mature projects.

Choose a mode as follows:

- No repository, folder, code, or URL means default to **pre-build strategy**.
- Any repository, folder, code excerpt, GitHub URL, or "I am building..." language means default to **mid-build review** unless the user says the project is finished, deployed, or ready for final review.
- If the user says the project is finished, deployed, in production, launch-ready, or ready for final review, use **post-build review**.

If a mid-build or post-build request provides only a description and no repository or code, proceed as an **advisory review from description**. Say that file-level findings require a repository or code sample. Do not pretend that you inspected local files.

## Hard Gates

- Treat the skill as read-only by default.
- Do not make a confident recommendation until you inspect the available evidence or state what evidence is missing.
- Do not recommend a stack because it is popular. Link each recommendation to the project's constraints, ecosystem fit, user skills, deployment path, and maintenance cost.
- Do not claim that a comparable is active, popular, secure, used in production, or better without evidence.
- Do not invent repositories, star counts, update dates, benchmark numbers, vulnerabilities, production adoption, or ecosystem norms.

## Permission Boundaries

The agent may:

- inspect repository structure and files that affect its architecture
- run read-only shell commands
- summarize project design and quality signals
- use available browsing/search tools for public references
- produce project strategy, stack recommendations, architecture options, and review reports

The agent must ask before it:

- modifying files
- installing dependencies
- running scripts that may change state
- running migrations, seeders, code generators, or package publish commands
- committing, pushing, opening issues, creating pull requests, or creating releases
- deleting files or changing configuration

## Safety and privacy

Do not read, print, summarize, or expose secrets from files such as:

- `.env` or `.env.*`
- `*.pem`, `*.key`, `id_rsa`, or SSH keys
- `credentials.json`, `secrets.*`, token files, or private config files
- production dumps, private certificates, or local auth/session stores

If sensitive files are detected, report only that they exist and recommend secure handling. Prefer file discovery commands that exclude dependency folders, build outputs, version-control metadata, and likely secret files.

## Workflow

Follow the checklist in order. Skip a step only when it is impossible or irrelevant, and say why.

1. **Frame the project** - identify the product goal, target users, core workflows, project stage, constraints, scale expectations, team/user skill level, deadline, budget, deployment target, and must-have integrations.
2. **Inspect existing evidence** - if a repository, folder, or URL exists, inspect README/docs, manifests, entry points, architecture notes, tests, CI, deployment config, and key source files. If no repository exists, use the user's description as the source of truth and list assumptions.
3. **Find useful references** - find credible comparable projects, official templates, reference architectures, standards, libraries, frameworks, and recent ecosystem guidance.
4. **Extract decision criteria** - decide what matters most for this project: speed of build, correctness, UI quality, scalability, cost, portability, security, extensibility, AI-navigability, hiring/community, or operational simplicity.
5. **Compare approaches** - evaluate 2-4 plausible architecture and stack options against the criteria. Include tradeoffs, migration risk, maturity, deployment fit, and when each option would be wrong.
6. **Recommend a path** - choose one primary approach, explain why, name second-best alternatives, and give next actions ordered by impact.
7. **Adapt to project stage** - for pre-build, produce a build strategy; for mid-build, produce recommended changes; for post-build, produce a review and improvement roadmap.

Before finalizing, run a quick self-check:

- Did the recommendation depend on actual project constraints rather than generic popularity?
- Did every "active", "maintained", "popular", or "production-ready" claim have evidence and an exact visible date or adoption signal?
- Did any section sound like a normal code review when no repo/code was inspected?
- Did the answer include when the recommended approach would become the wrong approach?

## Local Inspection Guidance

Use the fastest available read-only tools. Prefer `rg --files` for file discovery. If unavailable, use the platform's normal file listing tools.

Useful evidence to inspect:

- README, docs, ADRs, architecture notes, design notes
- manifests such as `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, `pom.xml`, `Gemfile`, lock files
- entry points such as `main.*`, `index.*`, `app.*`, `server.*`, `cli.*`
- route/controller/API definitions
- domain/service modules
- data models, schemas, migrations, query layers
- auth, permissions, secrets handling, validation, serialization
- test directories, fixtures, CI workflows, lint/typecheck config
- deployment and runtime config such as Docker, compose, infra, or platform files

Do not read every file unless the project is tiny. Sampling should be purposeful, and findings should cite files or commands as evidence.

## External research rules

Use the available web browsing/search tools if enabled. If browsing is unavailable, continue with local analysis and clearly state that external benchmarking was not performed.

For each external reference, record:

- URL
- visible last update date or maintenance signal, if available
- star count, package downloads, official status, or adoption signal, if available
- why it is relevant
- limits of the comparison

Prefer primary sources: repository pages, official documentation, release pages, framework templates, standards, maintainer-written case studies, and benchmark methodology pages. Be cautious with blogs, rankings, and "best X" lists unless they provide concrete evidence.

Freshness rules:

- Use exact dates when discussing updates, releases, maintenance, or "recent" guidance.
- Do not say "as of 2025", "current", "latest", "active", or "maintained" unless browsing or local git metadata verifies it.
- Treat star counts, package downloads, release dates, and last commit dates as time-sensitive. Include "visible at time of review" or the observed date when useful.
- If a comparable inspired the recommendation but uses a different current stack than expected, say that explicitly instead of flattening it into an older/simple version.

Comparable selection:

- Include at least one direct domain comparable when available.
- Include one official template/reference architecture when it would change stack or architecture decisions.
- Include one contrasting heavier or lighter alternative when it clarifies why the recommendation is not merely preference.

## Evaluation criteria

Assess the project or proposed approach across these dimensions when relevant:

- **Product fit** - whether the approach matches the intended user, workflow, and project stage.
- **Architecture** - boundaries, dependency direction, data flow, extensibility, and whether important concepts have clear homes.
- **Tech stack fit** - framework maturity, ecosystem support, deployment path, hiring/community, learning curve, performance needs, and maintenance cost.
- **Build speed** - how quickly the user can get to a useful working version without creating avoidable rework.
- **Correctness and reliability** - validation, error handling, edge cases, transactions, concurrency, and failure modes.
- **Security and privacy** - auth, authorization, secrets hygiene, input handling, dependency risk, and sensitive data handling.
- **Developer experience** - setup path, scripts, docs, CI, static checks, test feedback loops, and deploy clarity.
- **Scalability and operations** - cost, observability, scaling model, data growth, background jobs, queues, caching, and rollback strategy.

Match the recommendation to the project. A weekend prototype, hackathon app, internal tool, student project, open-source library, and production service do not need the same level of planning or review.

## Output contracts

Use the contract that matches the operating mode.

### Pre-Build Strategy

```md
## Project Approach: <Project Name>

### TL;DR

<Recommended approach and why.>

### Project Frame

<Goal, users, constraints, assumptions, success criteria, and evidence status.>

### Comparable Projects and References

1. **<Name>** - <URL>; <maintenance/adoption signal>; <why relevant>; <limits>.

### Recommended Stack

<Frontend, backend, data, auth, hosting, testing, observability, and any key libraries.>

### Architecture Direction

<How the project should be structured. Include a Mermaid or ASCII diagram when helpful.>

### Alternatives Considered

1. **<Option>** - <when it is better, when it is worse>.

### Build Plan

1. <First useful vertical slice>
2. <Next slice>
3. <Hardening/deploy/testing step>

### Risks and Unknowns

- <What could change the recommendation.>

### References

- <URL>
```

### Mid-Build or Post-Build Review

```md
## Project Approach Review: <Project Name>

### TL;DR

<Verdict, most important change, and what to keep.>

### Project Summary

<What it appears to do, who it serves, current stack, architecture shape, and maturity.>

### Evidence Reviewed

- Commands run: <short list>
- Files inspected: <short list of the most important files>
- External references: <count or "not performed">
- Evidence status: <local repository inspected | description only | GitHub URL only | mixed>

### What Is Working

- <Only real strengths, with evidence.>

### Comparable Projects or Benchmarks

1. **<Name>** - <URL>; <maintenance/adoption signal>; <why comparable>; <limits>.

### Gap Analysis

<Specific gaps between this project, its goals, and credible comparables or ecosystem practice.>

### Recommended Changes

#### High Priority

1. **<Change>** - <why, where, and expected impact>

#### Medium Priority

1. **<Change>** - <why, where, and expected impact>

#### Low Priority

1. **<Change>** - <why, where, and expected impact>

### Stack and Architecture Verdict

<Keep, adjust, or reconsider. Name tradeoffs and migration cost if relevant.>

### Risks, Assumptions, and Unknowns

- <What could change the verdict.>

### References

- <URL or local file reference>
```

Limit high-priority items to five. Keep the report direct and useful. Do not list every possible improvement.

## Failure handling

- **No accessible files** - ask for a path, archive, GitHub URL, or a short project description.
- **Idea only** - proceed in pre-build mode using assumptions, and call out the top questions that would change the recommendation.
- **GitHub URL only** - inspect public README, file tree, manifests, and key files through available browsing or a temporary read-only clone. Do not assume private access.
- **Tiny or empty project** - focus on project framing, stack choice, setup, basic structure, and first useful vertical slice.
- **Monorepo** - ask for the target package or app, or map the top level and identify candidates for deeper review.
- **Non-code project** - review organization, conventions, automation, data quality, docs, and maintainability instead of code architecture.
- **External research blocked** - say so and proceed with local evidence and general engineering judgment only.

## Review discipline

- Lead with evidence, not opinion.
- Separate "optimal for this project" from "popular in general."
- Reference actual files, commands, and sources for important claims.
- Make tradeoffs explicit: speed, complexity, cost, scale, hiring/community, portability, and maintenance.
- Offer concrete next moves, not abstract advice.
- Respect the user's goals. Make the project easier to build well. Do not judge it against an unclear standard.
