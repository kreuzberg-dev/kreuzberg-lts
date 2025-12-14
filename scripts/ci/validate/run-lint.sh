#!/usr/bin/env bash
#
# Run all linting and validation checks (check-only mode - no modifications)
# Used by: ci-validate.yaml - Run lint step
#

set -euo pipefail

echo "=== Running all lint checks in check-only mode ==="
task lint:check
echo "Lint checks complete"
