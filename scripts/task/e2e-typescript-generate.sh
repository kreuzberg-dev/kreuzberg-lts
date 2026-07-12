#!/usr/bin/env bash
set -euo pipefail

bash scripts/task/e2e-generate.sh typescript
poly fmt --fix e2e/typescript/tests
poly lint e2e/typescript/tests
