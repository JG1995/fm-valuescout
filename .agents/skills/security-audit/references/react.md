# React — Vite SPA security audit

Load this file when recon finds React under `src/`, a Vite client bundle, TanStack Router, TanStack Query, or Zustand. Read `references/universal.md` first.

This template defaults to a **client-rendered SPA inside a Tauri WebView** (desktop) or plain Vite dev in a browser. Every rule in the browser applies: attacker-controlled input, tampered requests, and full visibility into the JS bundle. Server enforcement (auth, authorization, prices, rate limits) must live in **Rust IPC commands** in the desktop default — never only in React.

For Tauri IPC, capabilities, CSP, and plugin risks, read `references/tauri.md`. For Vite env inline, source maps, and dev-server exposure, read `references/vite.md`. For test mocks and fixture leaks, read `references/testing.md`.

Forks that adopt Next.js, TanStack Start, or other SSR/meta-frameworks should also read framework-specific server patterns (Server Actions, middleware, RSC) — those are not covered here beyond a short fork note at the end.

Adapted for audit depth from [bogenc/audit-codebase](https://github.com/bogenc/audit-codebase) (Strix-derived vulnerability references, Apache-2.0), [raroque/vibe-security-skill](https://github.com/raroque/vibe-security-skill) (MIT), and OWASP / industry React SPA guidance (2025–2026).

## What React does and does not protect

React escapes strings in JSX text children and most attributes by default. That prevents a large class of reflected XSS when user input is rendered as `{value}`.

React does **not** protect:

- Raw HTML (`dangerouslySetInnerHTML`, markdown with HTML pass-through)
- URL schemes in `href` / `src` (`javascript:`, `data:`)
- Direct DOM APIs (`innerHTML`, `document.write`) in hooks or refs
- Client-only auth, secrets in the bundle, or server authorization gaps
- Third-party scripts running in your origin

Report XSS only when untrusted input reaches a sink without encoding for that context. JSX interpolation alone is not a finding.

## Attack surface

| Layer | What attackers reach |
| --- | --- |
| **Bundle** | All `VITE_*` inlined values, route chunks, lazy-loaded feature code |
| **Router** | Path params, search params, hash, history state, `beforeLoad` guards |
| **Query cache** | Persisted or devtools-exposed API responses |
| **Zustand / storage** | `localStorage`, `sessionStorage`, IndexedDB — readable and tamperable by XSS or extensions |
| **DOM** | User content, markdown, URLs in `href`/`src`, third-party widgets |
| **Outbound HTTP** | `fetch` from `api-client`, Query mutations, user-supplied URLs |
| **OAuth callback** | `code`, `state`, tokens in URL fragment before scrubbing |
| **Service worker** | Request/response proxy when PWA is enabled |

There is no server-side React in the template default. Do not treat client route guards as authorization.

## Untrusted input (treat as attacker-controlled)

Unless proven otherwise at a trust boundary:

- URL: path params, search params, hash, `document.referrer`
- Browser storage: values previously written by the app (XSS or extensions can alter them)
- `postMessage` payloads and WebSocket messages
- API, CMS, GraphQL, and feature-flag responses rendered in UI
- Third-party script output and tag-manager injections

## Recon signals

Search scoped code and built artifacts when a production bundle is in scope:

```text
dangerouslySetInnerHTML|innerHTML|outerHTML|insertAdjacentHTML|document\.write|DOMParser
eval\(|new Function\(|setTimeout\(|setInterval\(.*['"]
localStorage|sessionStorage|AsyncStorage|indexedDB
jwt\.decode\(|atob\(.*token
import\.meta\.env\.VITE_|process\.env\.NEXT_PUBLIC_
__proto__|constructor\.prototype|merge\(|defaultsDeep|lodash\.merge
postMessage|addEventListener\(['"]message
navigate\(|redirect\(|href=.*search|location\.|window\.open|returnTo|redirectTo|next=
persist\(|createJSONStorage|partialize|ReactQueryDevtools
credentials:\s*['"]include|withCredentials:\s*true
marked|markdown-it|react-markdown|rehype-raw|allowDangerousHtml|sanitize:\s*false
createElement\(['"]script|serviceWorker\.register|trustedTypes\.createPolicy
target=_blank(?!.*noopener)
```

**Build artifacts** (when `dist/` or CI artifacts are in scope):

- `dist/assets/*.js` — grep for `sk_live`, `service_role`, `AKIA`, `ghp_`, `xoxb-`, `Bearer `
- Source maps (`*.map`) shipped beside chunks — route names and internal paths leak
- Chunk filenames map to routes when using route-level `lazy()`
- `public/index.html` — third-party `<script>` without `integrity`

### Dev-only paths

MSW (`src/testing/mocks/`), `VITE_*` dev proxies, and Playwright fixtures must not ship in production bundles or env. Audit that production entry paths do not import test setup or dev-only mocks.

**Supply chain** — `pnpm-lock.yaml` present; CI uses frozen install (`pnpm install --frozen-lockfile` or equivalent); advisories for React, router, Vite, sanitizers.

## Secrets and client env (`VITE_`)

Vite inlines every `import.meta.env.VITE_*` value into the client bundle at build time. Anyone can read them from `dist/assets/*.js`.

| Safe in `VITE_*` | Never in `VITE_*` |
| --- | --- |
| Public API base URL (`/api`, `https://api.example.com`) | Service-role keys, DB URLs, signing secrets |
| Stripe publishable key (`pk_*`) | Stripe secret (`sk_*`), webhook secrets |
| Supabase anon key, Firebase web config | Supabase `service_role`, OAuth client secrets |
| Public analytics IDs | LLM API keys, admin tokens |

Audit checks:

- `src/vite-env.d.ts` and `src/config/env.ts` — Zod schema should reject empty strings and block secret-shaped names (`*_SECRET`, `*_KEY` unless publishable).
- `.env.example` — placeholders only; no real credentials.
- `vite.config.ts` `define` — no secrets inlined via `define`.
- `public/config.json` or runtime config fetched without auth — same exposure as env vars.
- Grep `import.meta.env` for non-`VITE_` access in client code (should not compile into client paths).

## XSS — React sinks and context

### Primary sinks

| Sink | Pattern | Audit trace |
| --- | --- | --- |
| `dangerouslySetInnerHTML` | `{ __html: userHtml }` | Source → sanitizer (DOMPurify?) → sink |
| Rich text / Markdown | `react-markdown` with `rehype-raw`, `allowDangerousHtml` | Plugin config allows raw HTML |
| URL attributes | `href={userUrl}`, `src={userUrl}` | `javascript:`, `data:` schemes |
| Event handlers from data | `onClick={userFn}` or spread props from API | Attacker-controlled function refs rare; check prop spread |
| Third-party embeds | User-controlled script URLs, JSONP loaders | Supply-chain and gadget XSS |
| JSON-LD / inline scripts | `script.textContent = userData` without `JSON.stringify` | Breakout from string context into script |

### DOM and non-React paths

- Direct `innerHTML` / `document.write` in hooks, utilities, refs, or legacy bridges.
- `location.hash` / `search` → DOM without `textContent` or safe encoding.
- `postMessage` handlers that write payload into DOM or call `eval`-like APIs.
- SVG/MathML uploads or inline SVG with user-controlled attributes.
- `new DOMParser().parseFromString(untrusted, 'text/html')` followed by insertion.

### Context rules (from sink type)

- **HTML text** — React children OK; raw HTML needs sanitizer or no HTML pass-through.
- **Attributes** — quote values; block `javascript:` in URLs.
- **JS string** — never build code from user input; use `JSON.stringify` for embedding (including JSON-LD).
- **CSS** — avoid injecting user strings into `style` or unsanitized `url()`.

### Defenses to verify, not assume

- CSP with nonces or hashes — weak `unsafe-inline` does not stop DOM XSS from inline gadgets.
- Static SPAs often need hash-based CSP or edge headers; per-request nonces require SSR or edge injection.
- DOMPurify — check `ALLOWED_TAGS`, `ALLOWED_ATTR`, URI schemes; centralize sanitization.
- Trusted Types — custom policies that return unsanitized strings are bypasses.

### Hydration and SSR forks

If the fork enables SSR or Server Components: hydration mismatches and server-rendered HTML with unsanitized input can reintroduce XSS on the client. Re-run XSS traces on both server HTML and client hydration paths.

Fork: `@tanstack/react-query-next-experimental` streamed hydration had XSS (CVE-2024-24558) in versions before 5.18.0 — verify version if experimental Next integration is in scope.

## URLs, navigation, and open redirects

React and routers do not safely handle arbitrary URL strings in navigation. Validate any URL derived from query params, storage, or API before use.

| Pattern | Risk |
| --- | --- |
| `window.location = next` | Full open redirect |
| `navigate(search.redirect)` | Open redirect via router |
| `href={userUrl}` | `javascript:` or phishing URL |
| Path starting with `//` | Protocol-relative URL → external host (React Router CVE-2026-40181 class; validate regardless of router) |

**Safe redirect rules** (audit for a shared helper or equivalent logic):

- Allow only same-origin relative paths: must start with `/` but not `//`
- Reject `javascript:`, `data:` (except tightly scoped cases), and off-origin absolute URLs unless allowlisted
- Parse with `new URL(value, window.location.origin)` and compare `origin` or pathname prefix
- Fall back to a safe default (`/`, `/dashboard`) on failure

TanStack Router `beforeLoad` redirects and `navigate()` deserve the same validation as `window.location`.

## Routing — TanStack Router

| Risk | What to trace |
| --- | --- |
| **UI-only auth** | `beforeLoad` or component checks that hide routes but API still serves data |
| **Open redirect** | `navigate({ to: search.redirect })`, `window.location = param` without allowlist |
| **Sensitive routes in bundle** | Admin paths only hidden in nav — chunks still load if URL is known |
| **Search param injection** | Params passed to `fetch`, `innerHTML`, or `navigate` without validation |
| **ID in URL** | `/users/$userId` — client shows data; server must enforce ownership (IDOR on API) |

Router guards improve UX. They do not replace server authorization on every API call.

## Authentication and session (SPA)

Common AI-generated mistakes (high signal in vibe-coded apps):

| Anti-pattern | Why it fails | Fix direction |
| --- | --- | --- |
| `jwt.decode()` without verify | Forged payload | Verify on server; client holds opaque token only |
| Tokens in `localStorage` / Zustand | Any XSS steals session | HttpOnly `Secure` `SameSite` cookies for session |
| Auth only in route component | Direct API access bypasses UI | Enforce on API; optional client guard for UX |
| "Logged in" from client flag | `isAdmin` in Zustand or polluted prototype | Server role on every mutation |
| Passing full user row to children | Password hash, internal flags in props | Select fields server-side; minimal DTO to client |

**Token storage ladder** (prefer top):

1. HttpOnly `Secure` `SameSite=Lax` (or `Strict` when UX allows) session cookies — not readable by JS; pair with CSRF defenses on state-changing API calls.
2. Short-lived access token in memory (React state/module closure) — lost on refresh; still vulnerable to XSS while tab is open.
3. `sessionStorage` — slightly narrower than `localStorage` but still fully readable by any script on the page.
4. `localStorage` / Zustand persist — avoid for auth tokens and refresh tokens.

For SPAs using cookie sessions: state-changing `fetch` with `credentials: 'include'` needs CSRF tokens, `SameSite` cookies, or double-submit cookie patterns on the API. `SameSite=Lax` helps but is not universal (cross-site POST, some embed flows).

When auditing full-stack forks: API should verify `Origin` / `Referer` on cookie-authenticated mutations where applicable.

## OAuth and OIDC in SPAs

High-impact audit cluster when login uses external identity providers:

| Check | Why |
| --- | --- |
| **Dedicated callback route** | Isolate OAuth `code`/`state` handling — avoid landing on `/` or dashboard with third-party analytics loaded |
| **Scrub URL immediately** | `history.replaceState` to remove `code`, `state`, tokens from address bar before other scripts run |
| **Validate `redirect_uri`** | Server allowlist only — open redirect here leaks authorization codes |
| **Tokens not in HTML** | No access tokens in SSR HTML, bootstrap JSON, or error telemetry payloads |
| **BFF pattern** | Prefer backend-for-frontend that holds tokens server-side when architecture allows |

OAuth `redirect_uri` and client-side `next`/`returnTo` params are the highest-impact open-redirect vectors — they can chain into token theft, not just phishing.

## CSRF (cookie-authenticated SPAs)

| Signal | Check |
| --- | --- |
| `credentials: 'include'` or `withCredentials: true` | State-changing POST/PUT/DELETE without CSRF token or SameSite protection |
| Cookie session + cross-origin API | CORS misconfiguration plus credentialed requests |
| Double-submit missing | Custom header (`X-CSRF-Token`) or form token on mutations |

Pure bearer-token SPAs (no cookies) are less CSRF-exposed but remain fully XSS-exposed.

## TanStack Query

| Risk | Check |
| --- | --- |
| **Sensitive cache** | PII, tokens, or admin fields in `queryKey` data or dehydrated state |
| **Persisted cache** | `persistQueryClient` + `localStorage` — durable secret storage |
| **DevTools in production** | `ReactQueryDevtools` bundled and exposed |
| **Optimistic updates** | Client assumes mutation success without server validation |
| **Error leakage** | `onError` or UI shows raw API error bodies with stack traces |
| **Experimental SSR packages** | Patched versions for known hydration XSS advisories |

Trace: API response → `select` / component — confirm no over-fetching of fields never shown in UI.

## Zustand and client state

| Risk | Check |
| --- | --- |
| **Auth in store** | `isAuthenticated`, `role`, `permissions` without server check on actions |
| **Unsafe merge** | `Object.assign(state, apiPayload)` or deep merge of API JSON into store — prototype pollution and mass assignment |
| **Persist middleware** | Secrets or PII in `persist` to `localStorage` |
| **Missing `partialize`** | Persist writes entire store including tokens — use `partialize` to whitelist non-sensitive UI fields only |

Prototype pollution probes (client): payloads like `{"__proto__":{"isAdmin":true}}` or `constructor[prototype][isAdmin]=true` merged into options objects. Block `__proto__`, `constructor`, `prototype` keys; use `Object.create(null)` for config merges.

## API client layer (`src/lib/api-client.ts` pattern)

- **Credentials** — `fetch` with `credentials: 'include'` sends cookies cross-site; review CORS on API.
- **Base URL** — `VITE_API_URL` pointing to attacker-controlled host in misconfigured env.
- **Dynamic base URL** — URL from query or storage used as `fetch` target without allowlist (data exfil).
- **No client-side authorization** — client sends `userId` in body; server must not trust it.
- **SSRF** — only relevant if the SPA triggers server-side fetches; pure SPA SSRF is usually outbound from browser (less critical than server SSRF).

## Third-party scripts and supply chain

Third-party JavaScript runs with full privileges of your origin.

| Check | Finding |
| --- | --- |
| `<script src="https://...">` in `index.html` without SRI | CDN compromise = full site compromise |
| Tag managers loading arbitrary scripts | Unreviewed code execution |
| `latest` or unpinned CDN URLs | Unexpected code changes on deploy |
| Missing lockfile or `pnpm install` without frozen lockfile in CI | Non-reproducible builds, typosquat risk |
| No dependency audit step | Known CVEs in React, router, Vite, sanitizers |
| Risky `postinstall` scripts | Install-time execution surface |

Audit `package.json` / lockfile versions for router open-redirect patches and Query experimental packages when those deps appear.

## Security headers (static SPA host)

Often configured at CDN or static host, not in Vite — mark "verify at edge" when absent from repo.

| Header | Purpose |
| --- | --- |
| **Content-Security-Policy** | Limit script/style sources; defense-in-depth for XSS |
| **Strict-Transport-Security** | Force HTTPS |
| **X-Content-Type-Options: nosniff** | Reduce MIME sniffing on user uploads |
| **Referrer-Policy** | Limit leakage of path/query in Referer |
| **frame-ancestors / X-Frame-Options** | Clickjacking defense unless embedding is required |
| **Permissions-Policy** | Restrict powerful APIs (camera, geolocation) when not needed |

Static SPAs without SSR: strict CSP without `unsafe-inline` may require hash-based policies or moving inline boot scripts. Do not "fix" breakage by adding `unsafe-eval` in production without documented need.

## Clickjacking and external links

- Missing `frame-ancestors` / `X-Frame-Options` — sensitive actions embeddable in attacker iframe.
- `<a target="_blank" href={url}>` without `rel="noopener noreferrer"` — tabnabbing via `window.opener`.

## postMessage and cross-window messaging

- `postMessage(data, '*')` — any origin can send messages.
- Missing `event.origin` check before acting on payload.
- `eval(event.data)` or `innerHTML = event.data` — direct XSS.
- Treat payload as data; render through React text or validated schema (Zod).

## Service workers and PWA

When `navigator.serviceWorker.register` or Workbox appears in scope:

- Must be served over HTTPS (except localhost).
- Service worker can intercept and modify all same-origin requests — cache poisoning and persistent MITM-style behavior if attacker controls SW script.
- Verify update strategy and that SW script is same-origin and integrity-protected.

## File uploads and blob URLs

- User SVG/HTML served or rendered with wrong `Content-Type` — treat as active content.
- `URL.createObjectURL` + `<img src>` for SVG — script execution in some browsers.
- Client-only validation of file type — server must re-validate.
- Inline preview of uploaded HTML/SVG via `dangerouslySetInnerHTML` or `<iframe src={blob}>` without sanitization.

## AI / LLM in React UI

- LLM keys in `VITE_*` or fetched from client-visible config.
- Model output rendered with `dangerouslySetInnerHTML` or markdown with raw HTML.
- Prompt built from unsanitized user + page content without isolation.
- No usage caps on client-triggered AI endpoints (abuse and cost).

## Deployment exposure (SPA)

- Production `sourcemap: true` in `vite.config.ts` without restricted hosting — template uses dual behavior: `Boolean(TAURI_ENV_DEBUG)` when `TAURI_ENV_PLATFORM` is set (Tauri builds), otherwise `sourcemap: 'hidden'` for standalone `vite build`. Audit release CI for `TAURI_ENV_DEBUG` in production jobs.
- `index.html` or env leaking staging API URLs with weaker auth.
- Debug overlays or React devtools hooks enabled in production build.
- `.git` or backup files exposed on static host.

## Static audit methodology

1. **Map routes** — `src/app/routes/`, `routeTree.gen.ts`, e2e smoke paths.
2. **Trace user input** — search params, forms, API JSON, storage → sinks (DOM, storage, outbound URL).
3. **Role matrix** — if auth exists: unauthenticated vs user vs admin on each route **and** matching API (client-only matrix is incomplete).
4. **Bundle review** — one production build; grep secrets and route chunks.
5. **OAuth / redirect review** — callback route isolation, `next`/`returnTo` validation, `//` path bypass.
6. **Confirm vs suspect** — React text interpolation alone is not XSS; trace to sink.

Skip server-only classes (SQLi in SPA-only template, server SSRF) unless recon shows a backend or BaaS rules in the same scope.

## False positives

- React rendering user string as child text (not in attribute URL or `dangerouslySetInnerHTML`).
- `VITE_API_URL=/api` relative path with same-origin API.
- Route `beforeLoad` redirect to login when API also returns 401 (defense in depth).
- MSW handlers in `src/testing/` not imported in `main.tsx` production path.
- Zod-validated env with only public URLs in `VITE_*`.
- `target="_blank"` with `rel="noopener noreferrer"` on external links.
- CSP/HSTS configured only at CDN — not visible in repo; report as "verify at edge" not confirmed gap.

## Fork note — Next.js / meta-frameworks

If recon finds `app/`, Server Actions, `middleware.ts`, or RSC:

- Middleware is not sole auth (bypass history includes header tricks and path normalization gaps).
- Server Actions are public POST endpoints — validate, authenticate, authorize on every action.
- `__NEXT_DATA__` and RSC flight payloads can over-fetch PII to the client.
- CSP nonces require dynamic rendering or edge injection — not automatic for static export.
- Load server-side framework references when auditing those paths; this file stays client-React focused.

## Sources

| Source | License / type | Use in this file |
| --- | --- | --- |
| [bogenc/audit-codebase](https://github.com/bogenc/audit-codebase) | Apache-2.0 (Strix-derived refs) | XSS context rules, prototype pollution shapes, client bundle recon |
| [raroque/vibe-security-skill](https://github.com/raroque/vibe-security-skill) | MIT | `VITE_` exposure table, JWT/session footguns, AI assistant anti-patterns |
| [JetBrains React security spec](https://github.com/JetBrains/skills/blob/main/security-best-practices/references/javascript-typescript-react-web-frontend-security.md) | Reference | URL validation, CSP/SRI, postMessage, supply chain audit order |
| [WorkOS SPA security guide](https://workos.com/guide/security-threats-in-single-page-applications-and-how-to-defend-against-them) | Guide | Layered XSS defense, cookie vs memory tokens |
| [OWASP XSS / CSP cheat sheets](https://cheatsheetseries.owasp.org/) | OWASP | Encoding contexts, CSRF coupling with cookies |
| CVE-2024-24558, CVE-2026-40181 | Public CVEs | Query experimental hydration XSS; router `//` open redirect class |
