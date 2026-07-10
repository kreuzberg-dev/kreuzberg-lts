#!/usr/bin/env bash

set -euo pipefail

tag="${1:?Release tag argument required}"

echo "Finalizing release ${tag}..."

if ! gh release view "$tag" >/dev/null 2>&1; then
  echo "::error::Release ${tag} does not exist. Cannot finalize."
  exit 1
fi

current_draft=$(gh release view "$tag" --json isDraft --jq '.isDraft' 2>/dev/null || echo "false")

if [ "$current_draft" = "true" ]; then
  echo "Release ${tag} is in draft state. Publishing..."
  gh release edit "$tag" --draft=false
  echo "::notice::Published release ${tag} from draft state"
else
  echo "Release ${tag} is already published"
fi

echo "Release ${tag} finalized successfully"
