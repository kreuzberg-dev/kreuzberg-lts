#!/usr/bin/env bash

# Publish Ruby gems directly to RubyGems
#
# This script:
# 1. Ensures latest RubyGems is installed
# 2. Validates gem files
# 3. Publishes using 'gem push'
#
# This script assumes gems have been freshly built and are not corrupted
# by artifact transfer (unlike the rebuild script which tries to repair corruption)
#
# Environment Variables:
#   - RUBYGEMS_VERSION: Version being published (optional, for summary)

set -euo pipefail

artifacts_dir="${1:-.}"

# Change to artifacts directory
cd "$artifacts_dir" || {
	echo "Error: Cannot change to directory: $artifacts_dir" >&2
	exit 1
}

# Ensure we're using latest RubyGems when possible.
# Hosted runners install RubyGems via apt, which rejects in-place upgrades.
if ! gem update --system; then
	echo "::warning::Skipping RubyGems update; system RubyGems installation does not support self-update (apt-managed runner)." >&2
fi

# Find all gem files
shopt -s nullglob
mapfile -t gems < <(find . -maxdepth 1 -name 'kreuzberg-*.gem' -print | sort)

if [ ${#gems[@]} -eq 0 ]; then
	echo "No gem artifacts found in $artifacts_dir" >&2
	exit 1
fi

# Validate gem files before publishing
echo "Validating gem files..."
for gem in "${gems[@]}"; do
	echo "Validating $gem..."

	# Check if file is readable and non-empty
	if [ ! -f "$gem" ] || [ ! -r "$gem" ] || [ ! -s "$gem" ]; then
		echo "::error::Gem file is invalid (missing, unreadable, or empty): $gem" >&2
		exit 1
	fi

	# Check file type (gems should be gzipped tar archives)
	file_output=$(file "$gem" 2>/dev/null || echo "")
	echo "File type: $file_output"

	# Verify gem is valid using gem spec
	echo "Validating gem with gem spec..."
	if ! gem spec "$gem" >/dev/null 2>&1; then
		echo "::error::Gem file validation failed: $gem" >&2
		echo "File type: $(file "$gem")" >&2
		exit 1
	fi
	echo "✓ Gem file validation passed"
done

echo "All gem files validated successfully"
echo ""

# Publish gems to RubyGems
echo "Publishing gems to RubyGems..."
failed_gems=()
for gem in "${gems[@]}"; do
	echo "Pushing ${gem} to RubyGems"
	publish_log=$(mktemp)
	set +e
	gem push "$gem" 2>&1 | tee "$publish_log"
	status=${PIPESTATUS[0]}
	set -e

	if [ "$status" -ne 0 ]; then
		if grep -qE "Repushing of gem versions is not allowed|already been pushed" "$publish_log"; then
			echo "::notice::Gem $gem version already published on RubyGems; skipping."
			if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
				echo "Gem $(basename "$gem") already published; skipping." >>"$GITHUB_STEP_SUMMARY"
			fi
		else
			failed_gems+=("$gem")
		fi
	fi

	rm -f "$publish_log"
done

if [ ${#failed_gems[@]} -gt 0 ]; then
	echo "::error::Failed to publish the following gems:" >&2
	for gem in "${failed_gems[@]}"; do
		echo "  - $gem" >&2
	done
	exit 1
fi

if [ -n "${GITHUB_STEP_SUMMARY:-}" ] && [ -n "${RUBYGEMS_VERSION:-}" ]; then
	echo "Successfully published kreuzberg version ${RUBYGEMS_VERSION} to RubyGems" >>"$GITHUB_STEP_SUMMARY"
fi

echo "All gems published successfully"
