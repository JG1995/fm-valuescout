---
name: semantic-versioning
description: Read this before you assign, increment, or validate a version number. If the user asks about versioning, or if you need to decide whether a change is major, minor, or patch, read this skill first.
---

# Semantic Versioning

## Overview

Every version in this project must follow the Semantic Versioning 2.0.0 format. The version number tells you whether a release contains bug fixes, new features, or breaking changes.

Use this skill when you pick a version number, decide what part to increment, validate a version string, or understand precedence between two versions.

## Version format

A normal version must follow X.Y.Z where X, Y, and Z are non-negative integers. They must not contain leading zeroes.

X is the major version, Y is the minor version, and Z is the patch version.

Valid: 1.0.0, 2.1.3, 0.4.1, 10.20.30

Invalid: 01.2.3, 1.0, 1.0.0.0, 1.0.0-alpha (pre-release, see below)

## Increment rules

| Part      | Increment when...                                                                                           |
| --------- | ----------------------------------------------------------------------------------------------------------- |
| MAJOR (X) | The public API changes in a backward-incompatible way                                                       |
| MINOR (Y) | Backward-compatible functionality is added to the public API, or any public API functionality is deprecated |
| PATCH (Z) | Only backward-compatible bug fixes are introduced                                                           |

A bug fix is an internal change that fixes incorrect behavior.

Adding new public API surface without breaking existing callers is a minor change.

Removing or changing existing public API in a way that breaks callers is a major change.

When you increment MAJOR, reset MINOR and PATCH to 0.
When you increment MINOR, reset PATCH to 0.

### Major version zero

Version 0.y.z is for initial development. The public API can change at any time. Do not treat it as stable. Start at 0.1.0 and increment the minor version for each release.

### First stable release

Release version 1.0.0 when you define the public API. If the software is in production, if users depend on a stable API, or if you worry about backward compatibility, you should already be at 1.0.0.

## Pre-release versions

Append a hyphen and dot-separated identifiers after the patch version to mark a version as unstable:

```
1.0.0-alpha
1.0.0-alpha.1
1.0.0-rc.1
1.0.0-0.3.7
```

Identifiers must use only ASCII alphanumerics and hyphens [0-9A-Za-z-]. Numeric identifiers must not have leading zeroes.

A pre-release version has lower precedence than the associated normal version. Use pre-release versions to signal that the version may not satisfy its intended compatibility requirements.

## Build metadata

Append a plus sign and dot-separated identifiers after the patch or pre-release version:

```
1.0.0+20240301
1.0.0-alpha+sha.abc123
1.0.0+21AF26D3----117B344092BD
```

Identifiers follow the same character rules as pre-release. Build metadata is ignored when determining version precedence. Two versions that differ only in build metadata have the same precedence.

## Version precedence

To compare two versions, separate each into major, minor, patch, and pre-release identifiers in that order. Build metadata does not affect precedence.

Compare from left to right:

1. **Major, minor, and patch** compare numerically.
   `1.0.0 < 2.0.0 < 2.1.0 < 2.1.1`

2. **Pre-release vs. release.** When major, minor, and patch are equal, a pre-release version has lower precedence than the normal version.
   `1.0.0-alpha < 1.0.0`

3. **Two pre-release versions** with the same major, minor, and patch are compared by their dot-separated identifiers, left to right:
    - Digits-only identifiers compare numerically.
    - Identifiers with letters or hyphens compare lexically in ASCII sort order.
    - Numeric identifiers always have lower precedence than non-numeric identifiers.
    - A longer set of identifiers has higher precedence when all preceding identifiers are equal.

    `1.0.0-alpha < 1.0.0-alpha.1 < 1.0.0-alpha.beta < 1.0.0-beta < 1.0.0-beta.2 < 1.0.0-beta.11 < 1.0.0-rc.1 < 1.0.0`

## Validation

Use this regular expression to check a SemVer string:

```
^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$
```

Group 1 = major, Group 2 = minor, Group 3 = patch, Group 4 = pre-release, Group 5 = build metadata.

The expression works in JavaScript, Python, Go, Perl, PHP, and R.

## FAQ

### What if I release a breaking change as a minor version by mistake?

Fix the problem immediately and release a new patch version that restores backward compatibility. Do not modify the already-released version. Document the offending release so users know about it.

### What if I update my own dependencies without changing the public API?

This is a compatible change. Whether it is patch or minor depends on whether you updated to fix a bug (patch) or to add new functionality (minor).

### How do I handle deprecation?

Deprecate parts of the public API in a minor release. Update the documentation to tell users about the change. Keep the deprecated functionality in place for at least one minor release before removing it in a major release.

### Is "v1.2.3" a valid SemVer version?

No.

The "v" prefix is not part of the semantic version. Using "v" as a tag prefix (e.g. `git tag v1.2.3`) is fine, but the version itself is "1.2.3".

## What to avoid

- **Leading zeroes in numeric parts.** Version 01.2.3 is invalid.
- **Modifying a released version.** Every change to a released version must be a new version. Never alter a published release.
- **Bumping MAJOR without an incompatible API change.** Major versions communicate breakage to consumers. Only increment MAJOR when callers must change their code.
- **Using pre-release or build metadata when you mean a release version.** Pre-release versions sort lower and signal instability. Do not use them for stable releases.
