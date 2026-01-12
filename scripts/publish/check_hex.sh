#!/usr/bin/env bash
# Check if Elixir package version exists on Hex.pm
#   $1: Package version (required)

set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <version>" >&2
  exit 1
fi

version="$1"
url="https://hex.pm/api/packages/kreuzberg/releases/${version}"
max_attempts=3
attempt=1
http_code=""

while [ $attempt -le $max_attempts ]; do
  echo "Checking Hex.pm for kreuzberg@${version} (attempt ${attempt}/${max_attempts})" >&2

  http_code=$(curl \
    --silent \
    --show-error \
    --retry 3 \
    --retry-delay 5 \
    --connect-timeout 30 \
    --max-time 60 \
    -o /dev/null \
    -w "%{http_code}" \
    "$url" 2>/dev/null || echo "000")

  if [ "$http_code" = "200" ] || [ "$http_code" = "404" ]; then
    break
  fi

  if [ $attempt -lt $max_attempts ]; then
    sleep_time=$((attempt * 5))
    echo "Warning: Hex.pm check failed (HTTP $http_code), retrying in ${sleep_time}s..." >&2
    sleep "$sleep_time"
  fi

  attempt=$((attempt + 1))
done

if [ "$http_code" = "200" ]; then
  echo "exists=true"
  echo "Elixir package kreuzberg@${version} already exists on Hex.pm" >&2
elif [ "$http_code" = "404" ]; then
  echo "exists=false"
  echo "Elixir package kreuzberg@${version} not found on Hex.pm, will build and publish" >&2
else
  echo "Error: Failed to check Hex.pm after $max_attempts attempts (last HTTP code: $http_code)" >&2
  exit 1
fi
