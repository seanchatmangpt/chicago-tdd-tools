#!/bin/bash
# Documentation Coverage Check
# Checks for missing documentation in source code using cargo doc

set -uo pipefail

echo "📚 Checking Source Documentation Coverage"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run cargo doc and capture warnings
# Use --document-private-items to check all items (not just public)
# Filter for missing_docs warnings
DOC_OUTPUT=$(timeout 30s cargo doc --no-deps --all-features --document-private-items 2>&1)

# Count missing documentation warnings
MISSING_DOCS=$(echo "$DOC_OUTPUT" | grep -c "missing documentation" 2>/dev/null | head -1 || echo "0")
TOTAL_WARNINGS=$(echo "$DOC_OUTPUT" | grep -c "warning:" 2>/dev/null | head -1 || echo "0")

# Extract specific missing doc warnings
MISSING_ITEMS=$(echo "$DOC_OUTPUT" | grep "missing documentation" 2>/dev/null | head -20 || echo "")

# Convert to integer (handle newlines and ensure single number)
MISSING_DOCS=$(echo "$MISSING_DOCS" | tr -d '\n\r ' | sed 's/[^0-9]//g' | head -c 10)
MISSING_DOCS=${MISSING_DOCS:-0}
# Remove leading zeros but keep at least one digit
MISSING_DOCS=$((10#$MISSING_DOCS))

TOTAL_WARNINGS=$(echo "$TOTAL_WARNINGS" | tr -d '\n\r ' | sed 's/[^0-9]//g' | head -c 10)
TOTAL_WARNINGS=${TOTAL_WARNINGS:-0}
TOTAL_WARNINGS=$((10#$TOTAL_WARNINGS))

echo "📊 Documentation Coverage Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Missing documentation warnings: $MISSING_DOCS"
echo "Total warnings: $TOTAL_WARNINGS"
echo ""

if [ "$MISSING_DOCS" -gt 0 ]; then
    echo "⚠️  Items Missing Documentation:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$MISSING_ITEMS" | while IFS= read -r line; do
        # Extract file and item name from warning
        if echo "$line" | grep -q "missing documentation"; then
            echo "  - $line"
        fi
    done
    echo ""
    echo "💡 Tip: Add '///' doc comments to public items"
    echo "💡 Tip: Add '//!' doc comments to modules"
    echo ""
    echo "📖 See: docs/process/CODING_STANDARDS.md for documentation standards"
    echo ""
    exit 1
else
    echo "✅ All public items have documentation!"
    echo ""
    echo "📖 Documentation standards:"
    echo "  - Public functions: Use '///' for doc comments"
    echo "  - Public modules: Use '//!' for module documentation"
    echo "  - Examples: Include '# Examples' sections for complex APIs"
    echo ""
    exit 0
fi

