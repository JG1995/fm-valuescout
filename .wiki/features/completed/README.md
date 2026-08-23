# Completed Feature Records

This directory contains historical completion records from earlier workflows and complete schema 2 ledgers from the current workflow. Do not rewrite older records only to match the current ledger schema.

For a current schema 2 feature, move its ledger here during final close-out. Preserve its full structure instead of rewriting it into a second summary format.

Keeping the delivery plan intact preserves the exact inputs needed to recompute its Delivery fingerprint during final publication, release, and recovery:

- every PR branch, base, provider, template, merge method, required-check rule, and title;
- every commit packet and packet fingerprint;
- completed-work validation and review evidence;
- exact implementation and merge refs;
- the accepted Release block; and
- discoveries, deviations, documentation impact, and publication corrections.

During close-out:

1. Run full feature validation and independent review.
2. Apply and review bounded corrections.
3. Reconcile current-state documentation and TODO state.
4. Set the final PR's `Feature close-out` to `Current`.
5. Preserve the accepted Delivery fingerprint and Release block unchanged for a delivered feature. For abandonment, follow the rule below.
6. Move the complete ledger from `.wiki/features/active/` to this directory in the reviewed close-out commit.

Do not remove packet fields, earlier PR metadata, or fingerprint inputs before the final PR merges and the release phase completes. Do not create a repository-only follow-up commit to record the final PR's self-referential merge ref or an external release result. The PR URL, planned release target, and verified provider state are the durable external evidence.

For `Release intent: none`, the delivery workflow completes with `Completed — no release intended`. For another intent, it runs the exact command once and verifies the exact target against the synchronized final merge.

An abandoned feature also keeps its schema 2 ledger. Set status to `Abandoned`, mark unfinished PRs and commits `Removed — <reason>`, resolve retained completed refs, record the approved abandonment details, set release intent and all release fields to `none`, recompute and record the Delivery fingerprint after that developer-approved authority change, remove Active work pointers, and move the ledger here. Later work starts from a fresh plan.
