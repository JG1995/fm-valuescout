#!/bin/bash
set -euo pipefail

# ==============================================================================
# Project setup — runs on every container start (postStartCommand).
# All operations are idempotent and safe to repeat.
# ==============================================================================

# ---------------------------------------------------------------------------
# 1. Mark the workspace as a git safe directory
# ---------------------------------------------------------------------------
# Fixes "dubious ownership" errors when the container user differs from the
# host user that owns the mounted volume.
git config --global --add safe.directory "$(pwd)"

# ---------------------------------------------------------------------------
# 1a. Set git identity if not already configured
# ---------------------------------------------------------------------------
# Prevents "unable to auto-detect email address" errors on first commit.
# Override locally if you need a different identity per-repo.
if [ -z "$(git config --global user.email)" ]; then
  git config --global user.email "jonas@greve.com"
  git config --global user.name "Jonas Greve"
fi


# ---------------------------------------------------------------------------
# 1b. Initialize git repo if not already present
# ---------------------------------------------------------------------------
# pre-commit requires a git repository to install hooks into .git/hooks/.
# On first start from a fresh template checkout, .git may not exist yet.
if [ ! -d .git ]; then
  git init
fi

# ---------------------------------------------------------------------------
# 2. Install pre-commit hooks into the local repo
# ---------------------------------------------------------------------------
# Re-runs idempotently — no harm if hooks are already installed.
pre-commit install
