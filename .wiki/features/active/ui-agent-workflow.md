# UI Agent Workflow

## Status

Active

## Intent

Provide a safe, developer-only way for Codex to control the real Tauri application, inspect it visually and semantically, edit the React UI, and verify the hot-reloaded result against real Rust IPC and isolated SQLite data.

## User-visible behavior

- `./scripts/dev ui-agent` starts the real development application with an empty, migrated, temporary database.
- `./scripts/dev ui-agent --dump /absolute/path/dump.json` validates and ingests the supplied player dump into the temporary database before the controllable session becomes ready. The source dump remains read-only.
- A trusted Codex task uses the pinned `@hypothesi/tauri-mcp-server` integration to inspect the live WebView, click, type, scroll, resize, capture screenshots, read logs, and inspect real Tauri IPC.
- Vite hot reload keeps the control loop usable after a React edit.
- A manually invoked `$workflow-ui-polish` skill guides open-ended UI improvement against the running application, then runs repository validation and presents before/after evidence.

## Invariants

- The `tauri-plugin-mcp-bridge` dependency and plugin registration exist only when the `ui-agent` Cargo feature is enabled in a non-release build.
- Ordinary development and release builds do not register the bridge, enable its WebView configuration, grant its capability, or expose its WebSocket endpoint.
- The bridge binds explicitly to `127.0.0.1`; its upstream all-interface default is never used.
- Every UI-agent run uses a new temporary application-data directory.
- A supplied dump path must be absolute. Startup reads it through the existing Rust snapshot validator and ingest service.
- There is no live-database mode. Product mutations affect only the temporary database.
- Product code continues to use real Rust IPC and Rust-owned SQLite. The feature adds no product-facing test IPC commands and no WebView SQL path.
- The upstream Rust and Node packages are version-pinned. Dependency updates repeat the runtime and release-boundary checks.
- The upstream control surface is trusted developer tooling. Its arbitrary JavaScript and IPC-related tools do not grant authority to access FM, manage plugins, confirm external destructive actions, or change Git state.
- Screenshots, logs, temporary databases, and spike artifacts remain under ignored `.work/` or operating-system temporary storage.
- UI-polish work preserves behavior, accessibility, feature-import boundaries, and data ownership unless the developer explicitly broadens the task.
- Normal Git approval rules still govern commits, pushes, and history changes.

## Non-goals

- A custom MCP server, WebDriver client, session protocol, or desktop automation framework.
- A reduced wrapper around the upstream MCP tool set unless real use proves the broad surface unsafe or unusable.
- A scenario language or replacement for Playwright smoke tests.
- Visual snapshot baselines or pixel-diff approval.
- CI integration for native UI control.
- Product-facing automation commands or testing-only IPC.
- Live access to the developer's application database.
- FM process control, bridge installation, plugin management, or automatic Load Data actions.
- Automatic commits from `$workflow-ui-polish`.
- A maintained demonstration database or synthetic player-data generator.

## Current-state map

- Relevant components: `scripts/dev` owns the stable command surface; `src-tauri/src/lib.rs` wires Tauri plugins and startup; `src-tauri/src/db/mod.rs` resolves `app_data_dir/app.db`; `src-tauri/tauri.conf.json` starts Vite for `tauri dev`; `src-tauri/capabilities/default.json` owns ordinary app permissions; `.codex/config.toml` registers project MCP servers; `.agents/skills/` owns repository workflows.
- Data model: The application uses one SQLite file named `app.db`; migrations v1-v7 run when `db::open` opens the file. No schema change is required.
- Persistence and migrations: `db::resolve_db_path` derives the current path from Tauri `app_data_dir`. Rust owns all SQL and migrations. Rust tests already use temporary databases through `db::open`.
- Sample data: The repository tracks no SQLite database. `src-tauri/src/features/memory_read/fixtures/golden_dump_v5.json` is a valid schema-v5 dump with one player. It is sufficient for automated startup and IPC proof, but not representative UI-polish coverage.
- Existing behavioral assumptions: `pnpm tauri dev` is the only current full-stack development loop. `./scripts/dev smoke` drives Chromium against Vite with stubbed IPC and does not exercise the native WebView, Rust commands, capabilities, or SQLite.
- Architectural seams: Cargo optional dependencies and a `ui-agent` feature can gate Rust plugin wiring; Tauri `--config` can merge a UI-agent-only configuration overlay; a development-only application-data override belongs in shared `db/`; orchestration belongs under developer tooling rather than a product feature; Codex can start a project-scoped STDIO MCP server from `.codex/config.toml` in a trusted project.
- Upstream integration: `hypothesi/mcp-server-tauri` supplies the Node STDIO MCP server and Rust Tauri bridge. It exposes screenshots, DOM and accessibility snapshots, interaction, window management, logs, JavaScript execution, and IPC-related tools through a loopback WebSocket connection.
- Upstream limits: The bridge defaults to all-interface binding unless configured otherwise, has no application-level authentication in the inspected WebSocket path, exposes a broader tool set than originally planned, and uses an `html2canvas` fallback for Linux screenshots.
- Test ownership: Rust unit tests own database-path, dump validation, and snapshot ingest semantics; tooling tests own argument validation, temporary-profile isolation, startup failure, and cleanup; configuration inspection owns feature/release exclusion; a bounded real-environment check owns the upstream bridge, WSL WebView, HMR, logs, screenshot quality, and IPC proof.
- Authoritative validation commands: `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`; manual `./scripts/dev ui-agent` proof is required because the existing smoke suite cannot cover the native runtime.
- Likely reuse points: `scripts/dev` argument and prerequisite helpers; `db::open`; `snapshot::ingest::ingest_dump_file`; the tracked golden dump; Tauri debug-only log plugin wiring; `.work/` ignore rule; existing repository workflow-skill structure.
- Applicable repository patterns: Keep the frontend thin and product persistence Rust-owned; keep `lib.rs` as wiring only; use optional Cargo dependencies for feature-gated plugins; use a config overlay for a development flavour; keep orchestration in `scripts/dev` or a small tool module; use project `.codex/config.toml` for trusted local MCP configuration; keep disposable artifacts under `.work/`.

## Feature architecture (this feature)

The feature is developer tooling, not a new product feature folder.

1. A disposable spike proves that the pinned `mcp-server-tauri` packages can control the real WSL Tauri development application, remain usable through Vite hot reload, expose useful logs and screenshots, and execute one real IPC query against an isolated database.
2. The isolated runtime adds an optional `ui-agent` Cargo feature, debug-only bridge registration, explicit loopback configuration, and a Tauri configuration overlay for bridge-only WebView settings and capabilities. The launcher creates the temporary profile, optionally seeds it from an absolute `dump.json` through the existing Rust snapshot ingest service, and owns application cleanup.
3. Codex starts the pinned upstream STDIO MCP server from `.codex/config.toml`. Its `driver_session` tool connects to the loopback bridge. The repository does not add a second MCP implementation, WebDriver layer, session metadata format, or wrapper protocol.
4. The UI-polish skill consumes the upstream tools. It owns the exploration and design loop, while existing coding, UI-design, validation, review, authority, and Git contracts remain authoritative.

The spike may change provider-specific startup details or the exact capability overlay. It must not weaken temporary-profile isolation, release exclusion, loopback binding, product boundaries, or the two-mode command contract.

## Uncertainty register

### Known

- The current Playwright suite cannot validate the real WebView, Rust IPC, SQLite, Tauri capabilities, or bridge behavior.
- `hypothesi/mcp-server-tauri` provides a Tauri v2 Rust bridge plus a Node STDIO MCP server with the required inspection, interaction, screenshot, log, window, and IPC capabilities.
- The bridge can be configured for `127.0.0.1`; its default configuration binds to `0.0.0.0`.
- The inspected upstream WebSocket path accepts and dispatches commands without an application-level authentication handshake. Loopback, debug-only compilation, trusted-project use, and temporary data therefore form the safety boundary.
- The upstream server exposes arbitrary JavaScript and IPC-related tools in addition to the narrower actions originally planned. In version 0.12.0, its advertised command executor does not dispatch application-defined Tauri commands; real product IPC remains accessible through WebView JavaScript and `window.__TAURI__.core.invoke`.
- Linux screenshots fall back to browser-side `html2canvas` rather than native WebView capture.
- Tauri supports merging a development-flavour configuration through `tauri dev --config`.
- Codex supports project-scoped STDIO MCP servers through `.codex/config.toml` in trusted projects.
- The repository has no tracked sample database. Its tracked golden dump contains one valid player.
- `.work/` is already ignored and is not project truth.
- `cargo clippy --all-targets --all-features` in `./scripts/dev check` will compile optional UI-agent Rust wiring during the gate.

### Assumptions

- WSLg and the installed WebKitGTK libraries can show the native application during the spike.
- The bridge capability and `withGlobalTauri` setting can remain confined to the UI-agent configuration overlay while ordinary and release builds stay unchanged.
- The pinned upstream MCP server can start from project config while the app is absent and connect after `./scripts/dev ui-agent` launches the bridge.
- Vite React Fast Refresh preserves or quickly restores a useful upstream control session.
- The upstream DOM snapshot provides enough semantic and accessibility information for open-ended UI-polish work.
- The upstream log tools and launcher output together provide useful frontend and Rust diagnostics.
- A developer-supplied realistic dump will normally seed UI-polish sessions. The one-player golden fixture is only a deterministic validation seed.

### Decisions

- Use one PR with two ordered commits. Adopting the upstream integration removes the custom MCP implementation commit.
- Keep the developer command at two modes only: empty migrated database or an absolute `dump.json` path ingested into a fresh temporary database.
- Pin the verified upstream Rust and Node package versions. Do not use an unpinned `npx -y` command.
- Register the upstream Node package as a project-local STDIO MCP server. Do not add a general HTTP service.
- Accept the upstream MCP tool surface for the initial trusted-development workflow. Do not build a filtering wrapper before repeated use demonstrates a concrete need.
- Explicitly configure the bridge for loopback and keep its dependency, registration, configuration, and capabilities out of ordinary and release builds.
- Do not create project-owned session metadata. The upstream `driver_session` lifecycle and loopback discovery own connection state.
- Use the tracked golden dump for automated seeding proof only. Do not present it as representative UI data or silently use a stale WSL database.
- Defer CI, scenarios, visual baselines, custom automation primitives, and synthetic sample-data generation until repeated use demonstrates a need.

### Unknowns

- Does the upstream server reconnect truthfully after a Rust-triggered application restart? — blocks: validation
- Does project-scoped Codex configuration need any extra startup instruction beyond opening a fresh task and invoking `driver_session` after the app launches? — blocks: validation

## Gating spike

Use `$workflow-spike` before `$workflow-build`.

**Question:** Can the pinned `hypothesi/mcp-server-tauri` integration give Codex reliable loopback-only control of the real WSL Tauri development application through Vite hot reload and one real Rust/SQLite IPC path without entering ordinary or release builds?

**Why inspection is insufficient:** Upstream documentation and source establish the components and tool surface, but they cannot prove WSLg/WebKitGTK behavior, configuration-overlay isolation, screenshot fidelity, hot-reload survival, log capture, or Codex startup in this repository.

**Success evidence:** A disposable experiment under `.work/spikes/` starts the debug application with a temporary database seeded from `src-tauri/src/features/memory_read/fixtures/golden_dump_v5.json`, connects the upstream `driver_session`, captures a useful screenshot, reads a DOM/accessibility snapshot, clicks and types, resizes the window, reads frontend and Rust diagnostics, edits a React file and observes the update without losing truthful control, and executes one real IPC query against the ingested player. It also proves the bridge binds only to `127.0.0.1` and that an ordinary or release build does not register it.

**Out of scope:** Production-quality launcher code, representative UI coverage, the UI-polish workflow, product UI changes, CI, visual baselines, and a custom MCP wrapper.

**Verdict routing:**

- `supported` or `conditional` with conditions compatible with the invariants: record exact package versions, startup commands, overlay/capability behavior, tool limitations, HMR behavior, screenshot path, and logs under Discoveries and replanning, then build PR 1 commit 1.
- `unsupported`: replan before implementation. The fallback is browser control for the editing loop plus a separate native screenshot verification step.
- `still uncertain`: keep commit 1 blocked and ask the developer which environment or upstream version to test next.

## Risks

### Broad unauthenticated control is exposed outside the intended boundary

- **Trigger:** The bridge uses its all-interface default, enters an ordinary or release build, or remains available against non-temporary data.
- **Consequence:** Another process can execute JavaScript or IPC against the application outside the intended trusted UI-polish session.
- **Mitigation:** Optional Cargo dependency, explicit `ui-agent` feature, debug-only registration, loopback-only configuration, UI-agent-only configuration and capability overlay, temporary database, pinned versions, and release inspection.
- **Proof:** Spike, commit 1 tests and manual inspection, and feature-complete review.

### UI-agent configuration leaks into normal builds

- **Trigger:** `withGlobalTauri`, bridge permissions, plugin dependency, or registration is added to the ordinary configuration path.
- **Consequence:** Normal development or shipped builds retain an unnecessary privileged surface or fail because optional permissions are unresolved.
- **Mitigation:** Use an explicit config overlay invoked only by `./scripts/dev ui-agent`; test featureless ordinary development and release builds; inspect resolved configuration and listening endpoints.
- **Proof:** Commit 1 validation and feature-complete review.

### Dump seeding diverges from product ingest

- **Trigger:** Startup parses or inserts the dump through separate tooling logic, or exposes the bridge before ingest succeeds.
- **Consequence:** The controlled application shows invalid or incomplete state that does not represent real product behavior.
- **Mitigation:** Require an absolute regular-file input; keep it read-only; call the existing Rust dump validator and snapshot ingest service after normal migrations; start the controllable window only after ingest succeeds.
- **Proof:** Commit 1 tests and real-environment validation.

### Screenshot evidence is visually incomplete on Linux

- **Trigger:** The upstream `html2canvas` fallback omits styles, assets, clipping, or native-window details needed for UI judgment.
- **Consequence:** The agent makes or approves visual changes from misleading evidence.
- **Mitigation:** Gate implementation on the spike's screenshot-quality check; compare the returned image with the visible WSL window; record unsupported rendering cases; replan to separate native verification if needed.
- **Proof:** Spike and UI-polish forward test.

### HMR or restart breaks the control loop

- **Trigger:** Fast Refresh replaces bridge-side injected state, or a Rust change restarts the application.
- **Consequence:** The agent edits blindly or acts through stale state.
- **Mitigation:** Prove React-edit behavior in the spike; require a fresh DOM snapshot after state changes; use upstream status/reconnect behavior truthfully; treat Rust restart as a new connection when required.
- **Proof:** Spike, commit 1 manual matrix, and commit 2 forward test.

### Open-ended agent actions exceed the intended UI task

- **Trigger:** The workflow treats the upstream JavaScript or IPC tools as permission for FM, plugin-management, destructive data, product-behavior, or Git actions.
- **Consequence:** State changes occur outside the developer's request.
- **Mitigation:** Encode explicit action boundaries in `$workflow-ui-polish`; keep the application on temporary data; preserve existing product, external-action, and Git approval rules.
- **Proof:** Commit 2 skill validation and fresh-context review.

### Upstream behavior or package compatibility drifts

- **Trigger:** The Rust plugin and Node server versions diverge, Tauri APIs change, or the upstream project changes tool or security behavior.
- **Consequence:** UI-agent startup, control, or safety assumptions fail while product builds remain healthy.
- **Mitigation:** Pin the versions proven by the spike, update them together, keep the integration small, and repeat manual native and release-boundary checks on upgrade.
- **Proof:** Commit 1 lockfiles, each upgrade review, and each real invocation.

## Walking skeleton

PR 1 commit 1 is the walking skeleton: an isolated real Tauri runtime plus the pinned upstream MCP integration that can inspect, act, capture an image, and prove real IPC. Commit 2 turns that capability into a repeatable UI-polish workflow.

## Delivery plan

### PR 1 — Add live Tauri UI agent workflow

**Status:** Active

**Provisional PR title:** `feat(ui-agent): add live Tauri UI workflow`

**Purpose:** Add the smallest safe native-app control path that lets Codex improve and verify the real UI without using live developer data or exposing automation in release builds.

**Depends on:** A `supported` or compatible `conditional` verdict from the gating spike.

**Merge boundary:** The complete PR is independently useful and trunk-safe. It provides the isolated upstream integration and its manual workflow together. A separate dependency-only PR would add coordination without delivering an independently useful capability.

#### Commit 1 — Integrate the isolated Tauri MCP runtime

**Status:** Completed

**Work:** Add a pinned optional `tauri-plugin-mcp-bridge` dependency behind the `ui-agent` Cargo feature and register it only in non-release UI-agent builds with explicit loopback configuration. Add a UI-agent-only Tauri configuration and capability overlay. Add a development-only application-data override. Extend `scripts/dev` with the two-mode launcher that validates an optional absolute dump path, creates a temporary profile, seeds its migrated database through the existing Rust snapshot ingest service, starts Tauri with Vite hot reload and the UI-agent overlay, and cleans up on exit. Pin and register the upstream Node STDIO MCP server in project Codex configuration.

**Out of scope for this commit:**

- A custom MCP server, WebDriver layer, session metadata format, or tool-filtering wrapper.
- UI-polish workflow instructions or product UI changes.
- CI integration, visual baselines, scenarios, or representative sample-data generation.
- Product IPC commands, schema changes, live database mode, or FM/plugin actions.

**Validation:** Tooling tests prove argument rejection, temporary-profile targeting, cleanup, and failed startup. Rust tests prove the database override resolves `app.db` under the temporary directory, valid dump seeding reuses the snapshot path, and invalid dump input prevents a controllable startup. Configuration checks prove the bridge dependency, registration, `withGlobalTauri`, and permissions are absent from ordinary and release builds and bind to `127.0.0.1` in UI-agent mode. `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke` pass. A fresh trusted Codex task starts the pinned STDIO server, exercises the required upstream tools against empty and golden-dump sessions, observes HMR, and proves isolated mutations and cleanup.

**Provisional commit:** `feat(ui-agent): integrate isolated Tauri MCP runtime`

##### Implementation profile

**Assigned implementer:** Sol — `gpt-5.6-sol` at `xhigh`

**Routing summary:** Capability Demand 8 routes to Terra before floors. The upstream bridge and MCP server remove the novel custom protocol and state-machine work, while the existing Rust snapshot ingest path supplies a tested persistence analogue. The unauthenticated arbitrary-JavaScript and direct-IPC surface changes a local authorization boundary, so the Sol High control-boundary floor overrides the raw route. Effort Demand 12 requires xhigh effort because the commit crosses Rust, Node, Tauri configuration, process lifecycle, IPC, SQLite, and manual WSL validation. Luna punch-up does not apply.

##### Review profile

**Assigned reviewer:** Sol — `gpt-5.6-sol` at `high`

**Context:** Fresh. The reviewer receives the commit contract, feature context, implementation packet, spike verdict, upstream version and source evidence, diff, validation, and repository access before implementation notes.

**Mandate:**

- Verify the bridge dependency, plugin registration, WebView configuration, and capability exist only in non-release UI-agent builds.
- Verify the bridge binds explicitly to `127.0.0.1` and no ordinary or release endpoint remains.
- Verify the temporary profile is unique per run and cleanup cannot target a broad or unresolved path.
- Verify a supplied dump is read-only and is ingested through the existing Rust validator and snapshot service before the controllable window starts.
- Verify the broad upstream JavaScript and IPC surface is documented and bounded by trusted-project, temporary-data, and workflow authority rules rather than misrepresented as a narrow tool set.
- Verify pinned Rust and Node package versions are compatible and project Codex configuration does not use unpinned package execution.
- Verify ordinary `pnpm tauri dev`, migrations, IPC, capabilities, and release behavior remain unchanged.

##### Implementation packet

###### Governing requirements and invariants

- Support only `./scripts/dev ui-agent` and `./scripts/dev ui-agent --dump /absolute/path/dump.json`.
- Treat the supplied dump as read-only seed input. Store all application mutations in the temporary database.
- Reuse `snapshot::ingest::ingest_dump_file`; do not add a second parser, SQL seed path, or product-facing import command.
- Pin the upstream Rust bridge and Node MCP server versions proven by the spike.
- Compile, configure, permit, and register the bridge only for the explicit non-release UI-agent build.
- Bind the bridge to `127.0.0.1`; never accept the upstream all-interface default.
- Use the upstream STDIO server and tool surface directly. Do not add project-owned session metadata or a wrapper protocol.

###### Existing patterns to follow

- `scripts/dev`: command dispatch, argument checks, prerequisite errors, scoped variables, and cleanup-safe shell style.
- `src-tauri/src/lib.rs`: debug-only log plugin and app-shell plugin wiring.
- `src-tauri/src/db/mod.rs`: path resolution and temporary-file database tests.
- `src-tauri/src/features/snapshot/ingest.rs`: existing dump validation, transactional ingest, active-save targeting, and role-score computation.
- `src-tauri/src/features/memory_read/fixtures/golden_dump_v5.json`: deterministic one-player seed for tests and spike proof only.
- `src-tauri/tauri.conf.json` and `src-tauri/capabilities/default.json`: ordinary Vite and capability configuration that UI-agent mode must not widen.
- `.codex/config.toml`: project-scoped MCP registration.
- `.gitignore`: existing `.work/` boundary.
- No repository analogue exists for this third-party native-control integration. Follow the verified spike behavior and pinned upstream contract.

###### Expected change surface

- **Likely modified:** `scripts/dev`, `vite.config.ts`, `package.json`, `pnpm-lock.yaml`, `.codex/config.toml`, `.codex/README.md`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/src/lib.rs`, and `src-tauri/src/db/mod.rs`.
- **Likely added:** A UI-agent Tauri config overlay, a UI-agent capability file or equivalent isolated permission configuration, and a small developer-tooling module with focused tests if Bash alone obscures lifecycle behavior.
- **Ownership boundaries:** The launcher owns the temporary profile and app process; Rust `db/` owns path resolution and startup seed ingest; `lib.rs` owns optional bridge registration; the upstream Node process owns MCP and driver-session state; the product frontend remains unchanged.
- **Do not change without replanning:** Database schema, snapshot ingest semantics, product IPC, FM bridge behavior, default app-data path, ordinary capabilities, release workflow, upstream tool surface, or the two-mode command contract.

###### State and data design

- The source dump is read-only input. The existing Rust snapshot service validates and ingests it after normal database migration.
- The temporary profile is the only application-data source of truth for the session.
- The launcher owns the application child process and cleanup. The upstream MCP process can exist while no application is running and reports connection state through `driver_session`.
- The bridge starts only after database migration and optional seed ingest succeed.
- A missing or invalid dump, failed ingest, plugin startup failure, or failed readiness check starts no usable controlled session and leaves the source untouched.
- The tracked one-player golden dump is never copied into normal application data implicitly. Tests or the spike pass its absolute path explicitly.

###### Expected interfaces

- `./scripts/dev ui-agent`
- `./scripts/dev ui-agent --dump /absolute/path/dump.json`
- One compile-time `ui-agent` Cargo feature with the optional bridge dependency.
- One UI-agent-only Tauri configuration overlay for the required WebView setting and capability.
- One development-only mechanism for `db::resolve_db_path` to use a launcher-supplied temporary application-data directory.
- One pinned project STDIO MCP registration that starts `@hypothesi/tauri-mcp-server` from installed dependencies.
- Upstream `driver_session` owns connection start, status, and stop behavior.

###### Execution order

1. Complete the spike and record verified package versions, commands, overlay behavior, tool limitations, and WSL evidence.
2. Add RED tests for argument/path validation, temporary-profile targeting, dump seeding, ingest failure, and cleanup.
3. Add the pinned optional Rust dependency, Cargo feature, and debug-only loopback bridge wiring.
4. Add the UI-agent-only Tauri configuration and capability overlay; prove ordinary configuration remains unchanged.
5. Add the development-only database-directory and optional dump-seed startup path with Rust tests.
6. Add the smallest testable launcher and `scripts/dev` command.
7. Pin the upstream Node package and register it in project Codex configuration.
8. Prove empty and golden-dump sessions through real upstream tools and real IPC, including HMR, screenshots, logs, isolated mutation, stop, and cleanup.
9. Run the full validation ladder and inspect an ordinary release-without-feature build.

###### Validation ladder

1. Targeted launcher tests for arguments, startup failure, and cleanup.
2. Targeted Rust database-path, dump-seeding, feature-gate, and loopback configuration tests.
3. Configuration and dependency inspection for ordinary, UI-agent, and release modes.
4. `./scripts/dev test`.
5. `./scripts/dev check`.
6. `./scripts/dev smoke` to prove browser smoke remains unchanged.
7. Ordinary development and release builds without `ui-agent`.
8. Fresh-task upstream MCP initialization and live tool matrix against empty and golden-dump sessions.
9. Manual WSL proof for HMR, screenshot fidelity, DOM/accessibility snapshot, logs, real IPC, isolated mutation, stop, restart, and cleanup.

###### Stop conditions

- Stop if the spike is not `supported` or compatible `conditional`.
- Stop if the bridge cannot bind only to loopback or cannot remain absent from ordinary and release builds.
- Stop if UI-agent-only WebView settings and capabilities cannot be isolated from normal configuration.
- Stop if dump seeding cannot reuse the existing Rust ingest path.
- Stop if the upstream server cannot provide useful screenshots, DOM inspection, input, resize, logs, and real IPC in WSL.
- Stop if safe use requires a custom authentication, MCP, or wrapper layer; return to planning before adding one.

###### Allowed discretion

- Local names, private helper decomposition, readiness polling, cleanup mechanics, and whether orchestration uses focused Bash or a small Node module, provided the contracts and validation remain intact.

###### Prohibited discretion

- Adding command modes, live database access, a second dump parser, public binding, release exposure, product IPC, schema changes, a custom MCP server, a filtering wrapper, CI, synthetic data generation, or generalized automation.

##### Escalation conditions

- **Increase effort when:** The selected ownership and upstream packages are correct but cleanup, configuration isolation, readiness, package wiring, or tests are incomplete.
- **Increase model capability when:** The implementation misunderstands the release/control boundary, accepts public binding, duplicates the upstream MCP layer, or bypasses snapshot ingest ownership.
- **Replan when:** The spike conditions change, bridge permissions cannot be isolated, the upstream package versions are incompatible, screenshot or HMR behavior is unusable, or a custom control layer becomes necessary.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-terra
    effort: xhigh
  implementer:
    model: gpt-5.6-sol
    effort: xhigh
    confidence: null
  capability_demand:
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 1
    semantic_risk: 3
    context_synthesis: 2
    total: 8
    luna_punch_up_applied: false
    hard_floor: sol-high-local-authorization-boundary
  effort_demand:
    implementation_breadth: 3
    branch_density: 2
    repository_discovery: 2
    validation_weakness: 3
    tool_coordination: 3
    adjustments: -1
    total: 12
  reviewer:
    model: gpt-5.6-sol
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 3
    hidden_interaction_complexity: 2
    validation_weakness: 3
    architectural_discretion: 2
    blast_radius: 3
    total: 13
    hard_floor: sol-high-local-control-and-release-boundary
  review_mandate:
    - Prove ordinary and release builds exclude the bridge dependency, configuration, capability, and endpoint.
    - Prove loopback-only control and temporary-data isolation.
    - Prove dump seeding reuses the existing Rust snapshot path before control becomes available.
    - Verify the broad upstream tool surface is represented and bounded truthfully.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - Ownership is correct but configuration, cleanup, or validation paths remain incomplete.
  escalate_model_when:
    - The implementation misunderstands release exclusion, loopback binding, or upstream ownership.
  replan_when:
    - Safe integration requires a custom control layer or changes the command or product boundary.
  adjudicator:
    model: gpt-5.6-sol
    effort: high
    invoke_when:
      - Reviewer and implementer disagree about the control or release boundary.
      - A high-severity public-binding or release-exposure finding remains disputed.
      - A correction would add a wrapper, custom MCP server, or product-facing command.
```

#### Commit 2 — Add the live UI-polish workflow

**Status:** Active

**Work:** Add a concise manually invoked `$workflow-ui-polish` repository skill. It connects through the upstream Tauri MCP tools, maps relevant routes and states organically, captures initial evidence, prioritizes the highest-value visual and interaction improvements, edits one cohesive batch at a time, re-inspects after hot reload, checks target viewports and accessibility paths, runs focused and repository validation, and presents before/after screenshots plus remaining concerns.

**Out of scope for this commit:**

- Product UI changes.
- Automatic commits or automatic feature-workflow invocation.
- Prescribed screenshot scenarios, a test DSL, visual baselines, or CI.
- Permission to change product behavior, architecture, data ownership, FM state, plugin state, or external destructive state.
- A custom MCP wrapper or replacement for upstream tools.

**Validation:** The skill passes the repository's skill validator, has correct trigger metadata, names the upstream live tools and preconditions, composes with `ui-design`, `coding-standards`, `minimalism`, and existing workflow rules without duplicating them, and is forward-tested in a fresh task against a realistic developer-supplied dump when available. The forward test captures before/after images, re-inspects after one safe UI edit, checks 1280x800 and 1600x900 plus keyboard, focus, overflow, errors, and logs, and stops before Git or external actions. `./scripts/dev check` and `./scripts/dev smoke` pass.

**Provisional commit:** `feat(workflow): add live UI polish workflow`

##### Implementation profile

**Assigned implementer:** Terra — `gpt-5.6-terra` at `high`

**Routing summary:** Capability Demand 5 would route to Luna before floors because the upstream tool contract and repository workflow patterns are established. The broad agent-action surface still requires safety and design judgment, so the Terra High agent-action floor applies. Effort Demand 4 would route to medium; the same floor raises the assigned effort to high. Luna punch-up does not apply.

##### Review profile

**Assigned reviewer:** Sol — `gpt-5.6-sol` at `high`

**Context:** Fresh. The reviewer receives the commit contract, feature context, implementation packet, skill artifact, upstream tool inventory, validation and forward-test evidence, and repository access before implementation notes.

**Mandate:**

- Verify the skill triggers only for intentional live UI-polish work and requires a running isolated session.
- Verify it uses the broad upstream JavaScript and IPC capabilities only for authorized UI inspection and interaction.
- Verify it gives useful open-ended design freedom without granting product-behavior, FM, plugin, external destructive, or Git authority.
- Verify it requires semantic and visual re-inspection after each cohesive edit batch and handles HMR or disconnect truthfully.
- Verify keyboard, focus, overflow, error, log, and both target viewport checks are explicit.
- Verify existing design, coding, validation, review, and Git contracts are reused instead of duplicated or weakened.
- Verify before/after evidence and remaining concerns are required without introducing scenarios or baselines.

##### Implementation packet

###### Governing requirements and invariants

- Manual opt-in only.
- Require a connected upstream `driver_session` against `./scripts/dev ui-agent`.
- Explore relevant routes and states organically; do not require predefined scenarios.
- Capture initial and final visual evidence.
- Make one cohesive UI batch at a time and re-inspect the hot-reloaded app.
- Preserve behavior, accessibility, architecture, feature imports, and data ownership.
- Check 1280x800 and 1600x900, keyboard navigation, focus, overflow, errors, and logs.
- Run focused tests, `./scripts/dev check`, and `./scripts/dev smoke`.
- Preserve normal Git and external-action approval rules.

###### Existing patterns to follow

- `.agents/skills/ui-design/SKILL.md`: design context, purposeful UI judgment, accessibility, and verification.
- `.agents/skills/workflow-build/SKILL.md` and `.agents/WORKFLOW.md`: scoped implementation, RED/GREEN where behavior changes, validation, review, and Git boundaries.
- `.agents/skills/minimalism/SKILL.md`: smallest cohesive improvement and no speculative framework.
- Existing `.agents/skills/workflow-*/SKILL.md`: concise frontmatter triggers, clear role boundary, Recallium hooks only when needed, and explicit output contract.
- Upstream Tauri MCP tool descriptions proven in commit 1; do not duplicate their schemas or invent aliases.
- System skill-creator validation pattern; repository workflow skills do not currently add `agents/openai.yaml`, so do not add it unless discovery shows repository skill loading requires it.

###### Expected change surface

- **Likely modified:** `.codex/README.md` or another narrow workflow index only if needed for discoverability.
- **Likely added:** `.agents/skills/workflow-ui-polish/SKILL.md` and only resources that repeated execution demonstrably needs.
- **Ownership boundaries:** The skill owns procedure and tool use; `ui-design` owns design judgment; existing workflow skills own implementation, review, and Git; the upstream package owns live control.
- **Do not change without replanning:** Product code, upstream MCP configuration or tool schemas, AGENTS.md standing contract, automatic commit permissions, or feature workflow semantics.

###### State and data design

- The workflow reads connection state through upstream `driver_session`; it does not infer readiness from files.
- Before/after screenshots and temporary notes stay under `.work/ui-agent/` when the upstream tool allows a file path; otherwise the MCP image response remains task evidence.
- Product edits remain ordinary workspace changes and follow existing validation and Git rules.
- A disconnected or reloading session pauses live actions, reports the state, and resumes only after a truthful reconnect and fresh DOM snapshot.
- A one-player golden-dump session can verify the workflow mechanics but cannot support claims about representative populated layouts. Prefer a developer-supplied realistic dump for the forward test.

###### Expected interfaces

- Manual invocation: `$workflow-ui-polish` plus a developer request such as "Explore the running app and improve the UI."
- Required live capabilities: upstream driver status, DOM/accessibility snapshot, screenshot, interaction, keyboard, window resize, logs, and reload or reconnect behavior proven in commit 1.
- Chat handoff states the improvements, validation results, before/after evidence, remaining concerns, and any behavior or architecture question left unchanged.

###### Execution order

1. Define realistic trigger examples, session preconditions, and explicit action exclusions.
2. Draft the smallest skill that composes existing guidance and names the upstream live-control loop.
3. Validate skill structure and metadata with the available validator.
4. Forward-test in a fresh task against a realistic dump when available, using one safe representative UI improvement.
5. Tighten only the instructions the forward test shows are missing or ambiguous.
6. Run repository validation without committing product UI changes from the forward test unless the developer separately requested them.

###### Validation ladder

1. Skill frontmatter and structure validation.
2. Focused prose, upstream tool-name, and command-reference inspection.
3. Fresh-task forward test against an isolated live session.
4. `./scripts/dev check`.
5. `./scripts/dev smoke`.
6. Manual confirmation that the workflow stops before Git, FM/plugin, or external destructive actions.

###### Stop conditions

- Stop if the skill needs new MCP tools, command modes, product IPC, or automatic Git authority.
- Stop if forward-testing would make an external destructive change or requires live FM/plugin access without approval.
- Stop and replan if safe open-ended exploration requires a wrapper, scenario framework, or custom automation layer.

###### Allowed discretion

- Skill prose, inspection ordering, prioritization heuristics, screenshot naming, and chat handoff shape within the fixed action and validation boundaries.

###### Prohibited discretion

- Adding product scope, automated commits, custom control tools, scenarios, visual baselines, CI, live databases, FM/plugin actions, synthetic data generation, or weakening accessibility and validation requirements.

##### Escalation conditions

- **Increase effort when:** The workflow boundary is correct but forward-testing reveals missed states, checks, tool semantics, or handoff evidence.
- **Increase model capability when:** The skill grants excess authority, treats upstream arbitrary execution as general permission, or conflicts with existing workflows.
- **Replan when:** The upstream tool set is insufficient, safe organic exploration requires a new abstraction, or the workflow cannot preserve existing Git and product boundaries.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-terra
    effort: xhigh
  implementer:
    model: gpt-5.6-terra
    effort: high
    confidence: null
  capability_demand:
    residual_ambiguity: 0
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 5
    luna_punch_up_applied: false
    hard_floor: terra-high-agent-action-boundary
  effort_demand:
    implementation_breadth: 1
    branch_density: 2
    repository_discovery: 1
    validation_weakness: 2
    tool_coordination: 1
    adjustments: -2
    total: 4
  reviewer:
    model: gpt-5.6-sol
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 3
    hidden_interaction_complexity: 2
    validation_weakness: 2
    architectural_discretion: 2
    blast_radius: 2
    total: 11
    hard_floor: sol-high-agent-authorization-boundary
  review_mandate:
    - Verify useful open-ended design freedom within existing authority boundaries.
    - Verify live re-inspection, accessibility, viewport, validation, and evidence requirements.
    - Verify broad upstream capabilities do not grant Git, FM, plugin, or external destructive authority.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The workflow is sound but forward-testing exposes missing states or checks.
  escalate_model_when:
    - The skill weakens authority boundaries or conflicts with existing workflows.
  replan_when:
    - Safe organic exploration requires new tools or generalized automation.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - Reviewer and implementer disagree about the workflow authority boundary.
      - A correction would add MCP tools, change Git approval, or broaden product scope.
```

## Active work

**PR:** PR 1 — Add live Tauri UI agent workflow

**Commit:** Commit 2 — Add the live UI-polish workflow

### RED test (active commit)

Given a manually invoked UI-polish request and a connected isolated Tauri session, a fresh task can follow the skill from initial semantic and visual inspection through one cohesive safe UI edit, hot-reload re-inspection, both target viewports, keyboard/focus/overflow/error/log checks, repository validation, and a before/after handoff without gaining Git, FM/plugin, external-destructive, product-behavior, or architecture authority. This catches an undiscoverable skill, stale-session actions, visual-only approval, missing accessibility checks, duplicated workflow policy, and authority creep.

### Expected outcome

The repository has a concise manually invoked `$workflow-ui-polish` skill that uses the runtime from commit 1 for open-ended UI judgment, requires truthful live re-inspection after each cohesive edit batch, preserves existing product and authority boundaries, validates the result, and presents before/after evidence plus remaining concerns.

### Explicit exclusions

- Do not make product UI changes as part of the workflow commit or its forward test.
- Do not add automatic commits, custom control tools, scenarios, visual baselines, CI, or representative sample data.
- Do not broaden authority to product behavior, architecture, live databases, Git, external destructive actions, or FM/plugin operations.

### Assigned profiles

- **Implementation:** Terra High — `gpt-5.6-terra` at `high`
- **Review:** Sol High — `gpt-5.6-sol` at `high`

### Current blockers

- None. A realistic developer dump remains preferable for populated-layout coverage, but the tracked golden dump can forward-test workflow mechanics without making representative-layout claims.

### Discoveries that may require replanning

- Replan if the skill needs new MCP tools, command modes, product IPC, automatic Git authority, a scenario framework, or a custom wrapper to support safe organic exploration.

## Discoveries and replanning

- The initial plan proposed WebdriverIO plus a project-owned STDIO MCP server and session metadata. The developer selected `hypothesi/mcp-server-tauri`, which already supplies the Rust bridge, persistent STDIO MCP server, screenshots, DOM snapshots, interaction, windows, logs, and IPC tools. The plan now uses that integration directly, removes the custom MCP and WebDriver layers, removes session metadata, and reduces the PR from three commits to two. The removed custom-control commit was assigned Sol max implementation and Sol High review; no equivalent custom implementation remains.
- The upstream integration simplifies implementation but broadens the trusted development surface. Its bridge defaults to all-interface binding, the inspected WebSocket path has no application-level authentication, and the MCP tools include arbitrary JavaScript plus IPC-related operations. Removing the custom server lowers the implementation from Sol max to Sol xhigh, but the local authorization boundary keeps a Sol implementation floor and Sol High review focused on loopback binding, release exclusion, temporary data, truthful capabilities, and dependency pinning.
- The initial runtime plan accepted an optional SQLite database and copied it into the temporary profile. The developer chose the smaller `--dump` contract because Windows and WSL use separate application-data roots and the repository already owns validated transactional dump ingest in Rust. Commit 1 seeds a fresh temporary database through that existing path.
- Repository inspection found no tracked SQLite database. The tracked schema-v5 golden dump contains one player and is suitable for automated seed and IPC proof only. Realistic UI-polish coverage requires an explicit developer-supplied dump, typically accessed from WSL through the mounted Windows filesystem. The plan does not add a synthetic database or silently reuse local WSL state.
- Real WebView e2e was previously deferred in `.wiki/BACKLOG.md` until stubbed smoke tests became insufficient. The developer has promoted a broader live UI-agent workflow, so the backlog item is superseded by this ledger rather than becoming a required CI e2e suite.
- The 2026-08-02 gating spike returned `conditional` for `tauri-plugin-mcp-bridge = 0.12.0` and `@hypothesi/tauri-mcp-server = 0.12.0` on the repository's Rust 1.97.1 and Tauri 2.11 toolchain. The bridge compiled, connected through the pinned upstream CLI, bound only to `127.0.0.1:9223`, and remained absent from a featureless release binary. An inline Tauri config overlay successfully confined `withGlobalTauri` and `mcp-bridge:default` to UI-agent mode.
- The spike produced useful Linux screenshots and DOM/accessibility snapshots, exercised click, typing, keyboard, resize, and console-log tools, and observed a React Fast Refresh update without losing the driver session. Rust startup and migration diagnostics remained available in the launcher output. A WSL locale rejection captured by the bridge caused Vite 8 console forwarding to crash while serializing the error, so WSL UI-agent mode sets Vite `server.forwardConsole` to `false`; ordinary and non-WSL UI-agent modes retain Vite's current behavior, and MCP console capture plus launcher output own diagnostics in WSL UI-agent mode.
- Bridge registration must occur dynamically after migrations and optional dump ingest. Builder-level registration exposed the control endpoint before startup preparation; delayed `app.handle().plugin(...)` registration started the endpoint only after a valid golden dump was queryable, while an invalid dump failed before any bridge listener existed.
- Version 0.12.0 advertises an IPC command executor but does not implement dispatch to application-defined Tauri commands. The spike proved real product IPC with WebView JavaScript calling `window.__TAURI__.core.invoke('search_players', ...)`. Commit 1 and the UI-polish workflow must describe this limitation truthfully rather than depend on the unsupported executor.
- The golden dump's source hash remained unchanged while a product IPC mutation changed only the temporary SQLite database. This satisfies the read-only seed and isolated-mutation boundary without copying or opening any developer database.
- Commit 1 landed as `8c0f209`. The pinned 0.12.0 integration passed the focused 11-test launcher/configuration suite, `./scripts/dev test` (156 tests), `./scripts/dev check` (198 Rust tests passed, 2 ignored), feature-specific Rust tests, a featureless release build, release-with-feature rejection, dependency/binary exclusion inspection, and a real golden-dump WSL session covering loopback connection, screenshot, DOM/accessibility inspection, resize, product IPC, isolated mutation, shutdown, and cleanup. The fresh Sol High review retained one WSL-scoping finding; the first fix round added host detection and a three-mode Vite configuration test, after which re-review was clean.
- Two repeated empty-session native retries stalled in the WSL WebKit environment before Tauri setup, migration, or plugin registration ran; automated empty-profile launcher coverage and existing migration tests passed. After the native run, Playwright smoke retained all 7 read-only checks but 5 click-based checks timed out awaiting a stable render frame; a fresh Sol High reviewer found no staged-patch causal path because smoke does not inherit launcher state and no product UI or smoke files changed. Repeat both native checks in a fresh WSL session during feature completion rather than representing them as passed.

## Completed work

| PR | Commit | Hash | Notes | Implementer | Reviewer | Deviations |
| --- | --- | --- | --- | --- | --- | --- |
| PR 1 | 1 | `8c0f209` | Added the isolated two-mode launcher, temporary Rust-owned database and dump ingest, pinned loopback MCP bridge/server, UI-agent-only Tauri overlay, WSL-scoped Vite workaround, project MCP registration, and focused tests. | Sol xhigh | Sol High; clean after one fix round | Empty native retry and click-based smoke checks were blocked by degraded WSL renderer state before staged application logic; evidence and required reruns are recorded above. |

## Final validation

- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Ordinary `pnpm tauri dev` without `ui-agent`.
- Release build without `ui-agent`, with no bridge dependency activation, plugin registration, UI-agent WebView setting, bridge capability, or listening endpoint.
- `./scripts/dev ui-agent` creates, migrates, controls, and removes an empty temporary profile.
- `./scripts/dev ui-agent --dump /absolute/path/dump.json` proves the real snapshot and role-score ingest path populates the temporary database and later mutations remain isolated to it.
- Golden-dump proof uses `src-tauri/src/features/memory_read/fixtures/golden_dump_v5.json`; representative UI-polish proof uses a developer-supplied realistic dump when available.
- Fresh trusted Codex task loads the pinned upstream STDIO MCP server and exercises driver status, screenshot, DOM/accessibility snapshot, click, typing, keyboard, scroll, window resize, logs, JavaScript, and IPC against the real app.
- HMR proof: a React edit appears in the controlled application; frontend reload and Rust-restart behavior are reported truthfully and reconnect when required.
- Lifecycle proof: no app, startup, ready, frontend reload, app restart, stop, crash, and second-run states.
- Loopback inspection proves the bridge listens only on `127.0.0.1`.
- Linux screenshot evidence is compared with the visible WSL window for material omissions.
- UI-polish forward test at 1280x800 and 1600x900 includes keyboard navigation, visible focus, overflow, errors, and frontend/backend logs.
- Before/after screenshots render in Codex and remain disposable.
- Fresh Sol High feature-complete review of end-to-end intent, read-only dump seeding, temporary-data isolation, release exclusion, upstream control boundary, HMR and IPC behavior, skill authority, and documentation accuracy.

### Feature review profile

- **Reviewer:** Sol High — `gpt-5.6-sol` at `high`, fresh context. This profile is fixed for every feature-complete review.
- **Mandate:** End-to-end runtime and control intent; read-only dump seeding through the existing snapshot path; temporary-data isolation; release and configuration exclusion; loopback-only bridge; pinned upstream integration; broad tool authority; real IPC, screenshot, logs, and HMR behavior; UI-polish authority, accessibility, and validation; temporary compatibility paths; documentation accuracy.

## Documentation impact

During implementation and final reconciliation:

- Update `.wiki/ARCHITECTURE.md` with the implemented developer-only runtime, exact compile and configuration gates, upstream MCP ownership, command behavior, temporary-data path, and distinction from Playwright smoke.
- Update `.codex/README.md` with project MCP discovery, pinned startup command, new-task or restart requirements, upstream `driver_session` precondition, and the `$workflow-ui-polish` entry.
- Update `README.md` only if the developer-facing setup belongs in the main setup path after the feature proves stable.
- Keep `.wiki/CONCEPT.md` unchanged because this is developer tooling, not product behavior.
- Do not add an ADR unless the spike forces a consequential difficult-to-reverse boundary beyond this ledger.
- At feature completion, archive this ledger and ensure the superseded Real WebView e2e backlog item remains removed rather than duplicated.
