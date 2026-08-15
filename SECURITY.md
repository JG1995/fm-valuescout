# Security policy

FM ValueScout is an early-alpha local desktop companion. Please do not disclose a suspected vulnerability in a public issue, discussion, pull request, or social media post.

## Report privately

Use GitHub's **Report a vulnerability** control on this repository's Security page. It opens a private report for the maintainer when private vulnerability reporting is enabled. Include a concise description, affected version or commit, reproduction steps, impact, and a safe proof of concept. Do not include FM saves, player dumps, database files, memory addresses, or access tokens unless the maintainer asks for them through the private report.

If the private-report control is unavailable, do not publish the details. Open a public issue containing only a request for a private reporting route, with no vulnerability information.

## Scope

Examples worth reporting include a path that exposes local FM data, a bridge operation that can write outside its documented action, an installer or release-workflow integrity problem, a secret committed to the repository, or code execution caused by untrusted input.

This project makes no response-time or remediation-time commitment. The maintainer will triage reports privately, confirm the scope, prepare a fix, and publish an advisory when it is safe to do so.
