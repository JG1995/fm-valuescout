# FMSuperScout — author permission

## Source

Reddit comment (r/footballmanagergames):

https://www.reddit.com/r/footballmanagergames/comments/1v73e6q/comment/ozv2tk6/?context=3

Participants:

- **GfxJG** (project author) asked permission to use / analyze the memory-reading plugin from the public GitHub repo for their own project.
- **mark17072** (FMSuperScout author) replied: “Yeah for sure, do whatever you like. That’s why it’s in GitHub.”

Screenshot also retained under Cursor workspace assets (2026-07-28).

## Effect on this repository

ADR-0016 originally forbade copying SuperScout source because the repo had **no license**. With **explicit author permission**, this project may:

- Port memory layouts, offsets, and algorithmic ideas into `bridge/` under our own structure
- Cite SuperScout as research provenance when pins stabilize

Still preferred:

- Keep our module layout (`Layouts/`, `Scanning/`, file protocol) rather than vendoring their plugin DLL wholesale
- Version pins by FM major.minor and fail closed on unsupported builds

## Related

- [ADR-0016](../decisions/0016-csharp-bepinex-fm26-bridge.md)
- Feature ledger: [fm26-memory-read](../features/active/fm26-memory-read.md)
- Upstream: https://github.com/mavarobli/FMSuperScout
