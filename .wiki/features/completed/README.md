# Completed Feature Records

This directory contains historical completion records from earlier workflows and complete schema 2 ledgers from the current workflow. Do not rewrite older records only to match the current ledger schema.

For a current schema 2 feature, move its ledger here during final close-out. Preserve its full structure instead of rewriting it into a second summary format.

Keeping the delivery plan intact preserves the exact inputs needed to recompute its Delivery fingerprint during final publication and recovery:

- every PR branch, base, provider, template, merge method, required-check rule, and title;
- every commit packet and packet fingerprint;
- completed-work validation and review evidence;
- exact implementation and merge refs; and
- discoveries, deviations, documentation impact, and publication corrections.

During close-out:

1. Run full feature validation and independent review.
2. Apply and review bounded corrections.
3. Reconcile current-state documentation and TODO state.
4. Set the final PR's `Feature close-out` to `Current`.
5. Preserve the accepted Delivery fingerprint. Preserve historical Release blocks unchanged.
6. Move the complete ledger from `.wiki/features/active/` to this directory in the reviewed close-out commit.

Do not remove packet fields, earlier PR metadata, or fingerprint inputs before the final PR merges and feature close-out completes. Do not create a repository-only follow-up commit to record the final PR's self-referential merge ref or an external release result. The PR URL and verified provider state are the durable external evidence.

PI_SETUP does not require release records. This repository uses an explicit [`create-release`](../../../.pi/skills/create-release/SKILL.md) invocation to prepare a separate release PR for the complete unreleased `main` range. Older records retain their historical Release blocks unchanged.

An abandoned feature also keeps its schema 2 ledger. Set status to `Abandoned`, mark unfinished PRs and commits `Removed — <reason>`, resolve retained completed refs, record the approved abandonment details, recompute and record the Delivery fingerprint after that developer-approved authority change, remove Active work pointers, and move the ledger here. Later work starts from a fresh plan.
