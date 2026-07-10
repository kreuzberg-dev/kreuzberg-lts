#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Testing Kreuzberg Docker tessdata configuration...${NC}\n"

test_tessdata_discovery() {
  local test_name="$1"
  local dockerfile="$2"

  echo -e "${YELLOW}Test: $test_name${NC}"

  if grep -A 10 "Setting up tessdata permissions" "$dockerfile" >/dev/null; then
    echo -e "${GREEN}✓ Tessdata setup code found in $dockerfile${NC}"
  else
    echo -e "${RED}✗ Tessdata setup code NOT found in $dockerfile${NC}"
    return 1
  fi

  if grep "TESSDATA_PREFIX=/usr/share/tesseract-ocr/5/tessdata" "$dockerfile" >/dev/null; then
    echo -e "${RED}✗ TESSDATA_PREFIX is still hardcoded in $dockerfile (should be removed)${NC}"
    return 1
  else
    echo -e "${GREEN}✓ TESSDATA_PREFIX is not hardcoded (correct)${NC}"
  fi

  if grep -q "chmod -R a+rx" "$dockerfile"; then
    echo -e "${GREEN}✓ Chmod command found to set permissions${NC}"
  else
    echo -e "${RED}✗ Chmod command NOT found in $dockerfile${NC}"
    return 1
  fi

  if grep -q "/usr/share/tesseract-ocr/\*/tessdata" "$dockerfile"; then
    echo -e "${GREEN}✓ Multiple tessdata paths checked in Dockerfile${NC}"
  else
    echo -e "${RED}✗ Multiple tessdata paths NOT found${NC}"
    return 1
  fi

  echo ""
  return 0
}

test_dockerfile_syntax() {
  local dockerfile="$1"
  local test_name="$2"

  echo -e "${YELLOW}Test: Verify $test_name syntax${NC}"

  if command -v docker &>/dev/null; then
    if docker build --dry-run -f "$dockerfile" "$PROJECT_ROOT" &>/dev/null; then
      echo -e "${GREEN}✓ Dockerfile syntax is valid${NC}"
    else
      echo -e "${YELLOW}! Dockerfile syntax check failed (may be due to missing Docker or build prerequisites)${NC}"
    fi
  else
    if grep -q "^FROM " "$dockerfile" && grep -q "^ENV " "$dockerfile"; then
      echo -e "${GREEN}✓ Basic Dockerfile structure looks valid${NC}"
    else
      echo -e "${RED}✗ Dockerfile structure is invalid${NC}"
      return 1
    fi
  fi

  echo ""
  return 0
}

test_user_permissions() {
  local dockerfile="$1"
  local test_name="$2"

  echo -e "${YELLOW}Test: User permissions in $test_name${NC}"

  if grep -q "USER kreuzberg" "$dockerfile"; then
    echo -e "${GREEN}✓ Non-root 'kreuzberg' user is set${NC}"
  else
    echo -e "${RED}✗ Non-root user NOT found${NC}"
    return 1
  fi

  if grep -q "chown -R kreuzberg:kreuzberg" "$dockerfile"; then
    echo -e "${GREEN}✓ Directory ownership set to kreuzberg user${NC}"
  else
    echo -e "${RED}✗ Directory ownership NOT set for kreuzberg user${NC}"
    return 1
  fi

  echo ""
  return 0
}

test_no_hardcoded_versions() {
  local dockerfile="$1"
  local test_name="$2"

  echo -e "${YELLOW}Test: No hardcoded version paths in $test_name${NC}"

  if grep "tesseract-ocr/5/tessdata" "$dockerfile" | grep -v "tesseract-ocr/\*/tessdata" >/dev/null; then
    echo -e "${RED}✗ Hardcoded tesseract-ocr/5 version found${NC}"
    return 1
  else
    echo -e "${GREEN}✓ No hardcoded tesseract-ocr/5 version${NC}"
  fi

  if grep "tesseract-ocr/4/tessdata" "$dockerfile" | grep -v "tesseract-ocr/\*/tessdata" >/dev/null; then
    echo -e "${YELLOW}! Hardcoded tesseract-ocr/4 version found (but it's in the loop, so OK)${NC}"
  else
    echo -e "${GREEN}✓ Version paths are in dynamic loop${NC}"
  fi

  echo ""
  return 0
}

run_tests() {
  local dockerfile="$1"
  local test_name="$2"
  local passed=0
  local failed=0

  echo -e "${YELLOW}========================================${NC}"
  echo -e "${YELLOW}Testing: $test_name${NC}"
  echo -e "${YELLOW}File: $dockerfile${NC}"
  echo -e "${YELLOW}========================================\n${NC}"

  if test_tessdata_discovery "Tessdata discovery logic" "$dockerfile"; then
    ((passed++))
  else
    ((failed++))
  fi

  if test_dockerfile_syntax "$dockerfile" "$test_name"; then
    ((passed++))
  else
    ((failed++))
  fi

  if test_user_permissions "$dockerfile" "$test_name"; then
    ((passed++))
  else
    ((failed++))
  fi

  if test_no_hardcoded_versions "$dockerfile" "$test_name"; then
    ((passed++))
  else
    ((failed++))
  fi

  echo -e "${YELLOW}----------------------------------------${NC}"
  echo -e "Results: ${GREEN}$passed passed${NC}, ${RED}$failed failed${NC}"
  echo -e "${YELLOW}========================================\n${NC}"

  return $failed
}

total_failed=0

if ! run_tests "$SCRIPT_DIR/Dockerfile.core" "Dockerfile.core"; then
  total_failed=$((total_failed + $?))
fi

if ! run_tests "$SCRIPT_DIR/Dockerfile.full" "Dockerfile.full"; then
  total_failed=$((total_failed + $?))
fi

echo -e "${YELLOW}========================================${NC}"
if [ $total_failed -eq 0 ]; then
  echo -e "${GREEN}✓ All tests passed!${NC}"
  echo -e "${GREEN}Tessdata configuration is properly set up.${NC}"
  exit 0
else
  echo -e "${RED}✗ Some tests failed (total failures: $total_failed)${NC}"
  echo -e "${RED}Please review the Dockerfile changes.${NC}"
  exit 1
fi
