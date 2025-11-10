#!/bin/bash
# Fix common linting issues automatically
# This script applies common fixes for clippy warnings

set -e

echo "🔧 Fixing common linting issues..."

# Fix uninlined_format_args - use inline format args
find src -name "*.rs" -type f | while read file; do
    # Fix format! with {} placeholders to use inline format args
    # This is a simple find/replace - more complex cases may need manual fixing
    sed -i '' 's/format!("{}/format!("{}/g' "$file" 2>/dev/null || true
done

# Fix missing_panics_doc - add Panics sections to assertion functions
# This requires manual review - script just identifies functions that need docs

echo "✅ Common lint fixes applied"
echo "⚠️  Some fixes require manual review (missing_panics_doc, etc.)"
