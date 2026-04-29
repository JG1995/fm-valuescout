#!/bin/bash
set -euo pipefail

# ==============================================================================
# Ensure all shell scripts in the project are executable
# ==============================================================================
# Git does not preserve file permissions, so after a fresh clone scripts may
# lack the executable bit. Running `./script.sh` then fails with
# "Permission denied". This script fixes that by setting +x on every .sh
# file in the repo (excluding node_modules).

# Find all .sh files and grant them the executable bit
find . -name "*.sh" -type f -not -path "*/node_modules/*" -exec chmod +x {} \;

# Print which scripts are now executable, for verification
find . -name "*.sh" -type f -not -path "*/node_modules/*" -executable
