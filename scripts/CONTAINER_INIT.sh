#!/bin/bash
set -euo pipefail

# ==============================================================================
# Container initialization — runs once when the devcontainer is first created.
# Only handles runtime setup that cannot be baked into the Docker image:
#   - script permissions (git doesn't preserve the executable bit)
#   - WSL2 zone-identifier cleanup
#
# System deps (apt-get, bun, uv, pre-commit) live in .devcontainer/Dockerfile.
# To add project-specific runtimes, edit the Dockerfile — not this script.
# ==============================================================================

# Resolve script locations relative to this file
SCRIPTS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPTS_DIR/.." && pwd)"

# ---------------------------------------------------------------------------
# 1. Fix script permissions
# ---------------------------------------------------------------------------
# Git doesn't preserve the executable bit, so freshly cloned .sh files may
# not be runnable with ./script syntax. This ensures they all have +x.
bash "$SCRIPTS_DIR/FIX_PERMISSIONS.sh"

# ---------------------------------------------------------------------------
# 2. Clean Windows Zone.Identifier files (WSL2 nuisance)
# ---------------------------------------------------------------------------
# WSL2 attaches :Zone.Identifier NTFS alternate data streams to files
# copied from Windows. These show up as separate files in Linux and
# can confuse tooling. Remove them on container creation.
bash "$SCRIPTS_DIR/DELETE_ZONE_IDENTIFIER.sh"

echo "Container initialization complete."
