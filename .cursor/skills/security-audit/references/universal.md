# Universal security checks

Load this file during every security audit. Stack-specific skills and references in `.cursor/skills/` add sinks, platforms, and framework patterns when recon finds matching technology.

## Core rule

Never trust the client. Validate or enforce every price, user id, role, subscription status, feature flag, and rate-limit counter on the server. If the value exists only in the browser, mobile bundle, or request body, an attacker controls it.

## Secrets and configuration

- Hardcoded API keys, tokens, passwords, or private keys in source or config committed to Git
- Secrets in client-exposed env prefixes (`NEXT_PUBLIC_`, `VITE_`, `EXPO_PUBLIC_`, and similar)
- `.env` or credential files not listed in `.gitignore`
- Debug flags, verbose errors, or stack traces enabled in production configuration
- Service-role or admin credentials reachable from client code or public bundles

## Authentication and session handling

- Missing authentication on state-changing or sensitive read endpoints
- `jwt.decode` or token parsing without signature verification
- Session or token storage in `localStorage`, `AsyncStorage`, or other client-readable stores when httpOnly cookies or secure storage are required
- Authentication enforced only in UI or middleware with no server-side check on the handler
- Password reset, magic link, or OAuth flows without expiry, binding, or replay protection

## Authorization and object access

- Resource fetched or updated by id without ownership or role check (IDOR)
- Function-level authorization missing (admin actions reachable by normal users)
- Mass assignment: bulk create or update from request body without an allowlist of fields
- Row-level security disabled, `USING (true)`, `allow: if true`, or equivalent open policies on managed databases

## Input and data access

- SQL or query built from string concatenation with user input
- Unsafe raw query helpers (`$queryRawUnsafe`, `.raw()`, string-built ORM filters)
- NoSQL or document queries that pass user objects directly into operators (`$where`, `$gt`, etc.)
- Missing validation at trust boundaries for types, length, and allowed values
- Deserialization of untrusted data (`pickle`, `yaml.load` without SafeLoader, native deserialize APIs)

## Web and client exposure

- Reflected or stored user input rendered into HTML or DOM without encoding (XSS)
- State-changing actions without CSRF protection when session cookies authenticate the user
- Open redirects built from request parameters without an allowlist
- User-controlled URLs passed to server-side HTTP clients (SSRF)
- File paths or upload names built from user input without canonicalization and root checks

## Payments and abuse

- Price or discount taken from the client request instead of server-side lookup
- Payment webhook handlers without signature verification
- Missing rate limits on auth, password reset, AI, email, or expensive endpoints
- Rate-limit counters stored only on the client or in tamperable session state

## AI and LLM integration

- LLM API keys exposed to the client or in public env vars
- Prompts built from untrusted user or external content without isolation or output handling
- No usage caps or billing limits on AI endpoints
- Model output rendered as HTML or executed without sanitization

## Deployment and exposure

- Source maps or `.git` exposed in production
- Missing security headers (CSP, HSTS, X-Content-Type-Options) when the stack supports them
- Production and development secrets sharing the same values

## How to report a finding

For each issue trace attacker-controlled input to the sink when possible. Separate **confirmed** (full path traced) from **suspected** (looks wrong but reachability not confirmed). Rank by real reachability: unauthenticated attacker path is Critical; theoretical issue behind multiple guards is Low.
