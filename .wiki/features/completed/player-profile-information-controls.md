# Player Profile Information Controls and Layout

## Intent

Deliver Linear issues JAY-5, JAY-8, and JAY-9 in one profile-focused PR: save-scoped hidden-information visibility, an FM-style attribute layout, and phase-specific role summaries.

## Delivered behavior

- Each player profile can reveal or conceal hidden information for the active app save. The preference persists across restarts, defaults to revealed, and applies to every player in that save.
- Concealment keeps current player information visible. It removes PA, projected values, potential role scores, Hidden and Personality values, and development actions that could disclose those values. Potential summary slots remain as concealed placeholders.
- Profiles have Outfield, Goalkeeping, Hidden, and Personality tabs. Outfield players default to Outfield. Players with GK familiarity of at least 15 default to Goalkeeping, which includes goalkeeper attributes, First Touch, Passing, Technique, Mental, and Physical; their Outfield tab shows the remaining Technical attributes and Set Pieces.
- The summary contains Current IP, Current OOP, Potential IP, and Potential OOP. Each selects the best non-null eligible role score for its phase. Ties retain catalog order and missing values render `—`.
- Profile panels size from the shell main area. A visible Load Data outcome banner cannot create nested page scrolling.

## Final architecture

- Migration v23 adds `saves.reveal_hidden_player_information` as a constrained `0|1` value with default `1`. `get_player` returns the active save's value and `set_player_hidden_information_revealed` writes an explicit state to that save.
- The profile route owns the mutation and invalidates `playerKeys.all` after success. Pending and error feedback is keyed by player UID and active save ID.
- The complete player DTO remains available to domain code. React owns the concealment render boundary because this preference is not access control.
- Canonical and legacy tab parsing, goalkeeper grouping, and phase-specific summary selection remain in profile presentation utilities. Search and Squad navigate without forcing a tab so the loaded profile can choose its player-sensitive default.

## Important decisions

- No ADR. The preference follows the existing save-owned persistence boundary and does not introduce a new architectural seam.
- No debug report. The two feature-review defects have focused regression coverage and do not require a reusable diagnostic procedure.

## Migration and operational implications

- Existing databases migrate to v23 with hidden information revealed. New saves receive the same default.
- The final PR also updates application icon assets and adds dismissible Load Data outcome banners. The banner only describes the save and context that initiated the request; it clears after a save-context switch.
- Fresh native Tauri/WebView validation and packaged icon rendering remain unverified in this environment.

## Validation

- `./scripts/dev format src e2e` passed with no remaining changes.
- `./scripts/dev test` passed: 37 files, 429 tests.
- `./scripts/dev check` passed: frontend checks and 401 Rust tests, with 2 ignored. Biome reported only its schema/CLI version notice.
- `./scripts/dev smoke` passed: 36 Chromium tests, including Load Data banner containment at 1280×800.
- `git diff --check` passed. `./scripts/dev mutate` is unsupported; `./scripts/dev bridge-test` was not applicable.
- The feature-complete review required one correction round. Review 1 found nested scrolling with a Load Data outcome and stale visibility feedback across a save switch. The correction added focused RED/GREEN coverage. Review 2 found no CRITICAL, HIGH, MEDIUM, or NITPICK findings and recommended acceptance.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
feature_close_out: current
branch: feature/player-profile-information-controls
base: main
provider: GitHub
template: .github/pull_request_template.md
merge_method: squash
required_check: check
final_feature_review_blocking: false
correction_rounds: 1
implementation_range: e82b24985af7d297abea95446e055e36c2616f17..28126911307050ca275b8e59cb5eaed89b577f90
exact_implementation_refs:
  - b9478779a6436b3274973d70cef798ac502d923d
  - 98ebc71826d20cd792d1580d1aa5eff12f5c7010
  - dc76f9724a8fdb1f3ab1bb3dfea1c0c3413ee8bd
  - 4f7481bb48ee81ce24e9f24de4ad7e26b536670d
  - 5da9fcf71cf5ab320f1df4893e0e07c4efbda6db
  - 8a9a6d413841a388cc07267adf24cdb9f9a79b14
  - ccd46e236f31b8e1f8da6ef62a39134d97ff8a1e
  - cf25d6d201a416b5f91943a28144b9a1c43ed54a
  - 762442896201f2c5dc36ddac525213caa212c2a5
  - f7dd612d1668c6ebef7c9cf38d6607daa99d1ec7
  - e25355e9a0923145163293aa8b8187395b047b2d
  - ed2db431cd575daff7f85ea66d76bcd6721794d9
  - 28126911307050ca275b8e59cb5eaed89b577f90
close_out_documentation_ref: Pending record
```

## Follow-up

Publish the final PR with the repository template. Record its URL and immutable merge reference only after those events occur.
