#!/usr/bin/env bash

set -euo pipefail

LOG_FILE="${1:-/tmp/build-log.txt}"

if [[ ! -f "$LOG_FILE" ]]; then
  echo "::error::Build log not found at $LOG_FILE"
  exit 1
fi

echo "📋 Scanning build log for issues..."
echo "---"

ISSUES=0

# Check for WARNING lines
if grep -inE '(WARNING|WARN)\b' "$LOG_FILE" | grep -v 'Expected warnings: 0'; then
  echo ""
  echo "::warning::Build log contains warnings (see above)"
  ISSUES=1
fi

if grep -inE '\bERROR\b' "$LOG_FILE"; then
  echo ""
  echo "::error::Build log contains errors (see above)"
  ISSUES=1
fi

if grep -i 'not found' "$LOG_FILE" | grep -iv 'expected'; then
  echo ""
  echo "::warning::Build log contains 'not found' references (see above)"
  ISSUES=1
fi

if grep -i 'unknown cross-reference' "$LOG_FILE"; then
  echo ""
  echo "::error::Build log contains broken cross-references (see above)"
  ISSUES=1
fi

echo "---"

if [[ "$ISSUES" -eq 0 ]]; then
  echo "✅ Build log is clean — no warnings or errors found."
else
  echo ""
  echo "❌ Build log contains issues. Please review the warnings/errors above."
  exit 1
fi
