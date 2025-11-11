#!/bin/bash
# Check for dog fooding violations in test files
# Detects:
# 1. Standard assertions (assert!, assert_eq!, assert_ne!) in test files
# 2. #[test] and #[tokio::test] attributes in test files
#
# Exit code: 0 if no violations, 1 if violations found

set -e

VIOLATIONS=0

echo "🔍 Checking for dog fooding violations..."

# Check for standard assertions in test files
echo "Checking for standard assertions (assert!, assert_eq!, assert_ne!)..."
STANDARD_ASSERTIONS=$(grep -r --include="*.rs" -E "assert!\(|assert_eq!\(|assert_ne!\(" tests/ src/ 2>/dev/null | grep -v "//!" | grep -v "stringify!" | grep -v "^Binary" | wc -l | tr -d ' ')

if [ "$STANDARD_ASSERTIONS" -gt 0 ]; then
    echo "❌ Found $STANDARD_ASSERTIONS instances of standard assertions in test files"
    echo "   Use library assertion macros instead: assert_ok!, assert_err!, assert_eq_msg!, assert_that!"
    grep -r --include="*.rs" -n -E "assert!\(|assert_eq!\(|assert_ne!\(" tests/ src/ 2>/dev/null | grep -v "//!" | grep -v "stringify!" | grep -v "^Binary" || true
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo "✅ No standard assertions found"
fi

# Check for #[test] attributes in test files
echo ""
echo "Checking for #[test] attributes..."
TEST_ATTRIBUTES=$(grep -r --include="*.rs" -E "^#\[test\]|^\s+#\[test\]" tests/ src/ 2>/dev/null | grep -v "//!" | grep -v "^Binary" | wc -l | tr -d ' ')

if [ "$TEST_ATTRIBUTES" -gt 0 ]; then
    echo "❌ Found $TEST_ATTRIBUTES instances of #[test] in test files"
    echo "   Use test! macro instead: test!(test_name, { /* AAA */ })"
    grep -r --include="*.rs" -n -E "^#\[test\]|^\s+#\[test\]" tests/ src/ 2>/dev/null | grep -v "//!" | grep -v "^Binary" || true
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo "✅ No #[test] attributes found"
fi

# Check for #[tokio::test] attributes in test files
echo ""
echo "Checking for #[tokio::test] attributes..."
TOKIO_TEST_ATTRIBUTES=$(grep -r --include="*.rs" -E "^#\[tokio::test\]|^\s+#\[tokio::test\]" tests/ src/ 2>/dev/null | grep -v "//!" | grep -v "^Binary" | wc -l | tr -d ' ')

if [ "$TOKIO_TEST_ATTRIBUTES" -gt 0 ]; then
    echo "❌ Found $TOKIO_TEST_ATTRIBUTES instances of #[tokio::test] in test files"
    echo "   Use async_test! or fixture_test! macro instead: async_test!(test_name, { /* AAA */ })"
    grep -r --include="*.rs" -n -E "^#\[tokio::test\]|^\s+#\[tokio::test\]" tests/ src/ 2>/dev/null | grep -v "//!" | grep -v "^Binary" || true
    VIOLATIONS=$((VIOLATIONS + 1))
else
    echo "✅ No #[tokio::test] attributes found"
fi

echo ""
if [ "$VIOLATIONS" -eq 0 ]; then
    echo "✅ All dog fooding checks passed!"
    exit 0
else
    echo "❌ Found $VIOLATIONS violation(s)"
    echo "   See above for details"
    exit 1
fi

