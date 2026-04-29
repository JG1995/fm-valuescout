#!/bin/bash
set -euo pipefail

# ==============================================================================
# Delete Windows :Zone.Identifier files from the project
# ==============================================================================
# When files are created or copied on a Windows/NTFS filesystem, Windows may
# attach a :Zone.Identifier alternate data stream (e.g. "file.sh:Zone.Identifier")
# marking the file as downloaded from the internet. Under WSL2 these appear as
# separate small files and can confuse diffing, linting, and packaging tools.
# This script finds and deletes them.

# Resolve the project root (one directory up from this script)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Count how many Zone.Identifier files exist (suppress errors for unreadable dirs)
COUNT=$(find "$PROJECT_ROOT" -type f -name "*:Zone.Identifier" 2>/dev/null | wc -l)

# Bail out early if there's nothing to clean
if [ "$COUNT" -eq 0 ]; then
    echo "No :Zone.Identifier files found."
    exit 0
fi

echo "Found $COUNT :Zone.Identifier file(s). Deleting..."

# Find and delete every Zone.Identifier file, printing each one as it's removed
find "$PROJECT_ROOT" -type f -name "*:Zone.Identifier" -print -delete 2>/dev/null

echo "Done. Deleted $COUNT file(s)."
