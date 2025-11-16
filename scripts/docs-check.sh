#!/bin/bash
# Documentation validation script
# Validates version numbers, build commands, links, examples, and style guide compliance

set -uo pipefail

ERRORS=0
WARNINGS=0
CURRENT_VERSION=$(grep '^version' Cargo.toml 2>/dev/null | cut -d'"' -f2 || echo '')
DOCS_DIR="docs"

if [ -z "$CURRENT_VERSION" ]; then
  echo "❌ ERROR: Cannot read version from Cargo.toml" >&2
  exit 1
fi

echo "📚 Documentation Validation" >&2
echo "Current version: $CURRENT_VERSION" >&2
echo "" >&2

# Function to report error
error() {
  echo "❌ ERROR: $1" >&2
  ERRORS=$((ERRORS + 1))
}

# Function to report warning
warning() {
  echo "⚠️  WARNING: $1" >&2
  WARNINGS=$((WARNINGS + 1))
}

# Step 1: Validate version numbers
echo "1️⃣  Checking version numbers..." >&2
VERSION_ERRORS=0
while IFS= read -r file; do
  # Check for version references that don't match current version
  # Allow planned version docs (v1.2.0-coverage-strategy.md)
  # Allow release notes files that match current version (RELEASE_NOTES_v1.2.0.md when version is 1.2.0)
  # Allow release announcement files (ANNOUNCEMENT_v1.4.0.md, GITHUB_RELEASE_v1.4.0.md)
  if [[ "$file" == *"v${CURRENT_VERSION}"* ]] && [[ "$file" != *"coverage-strategy"* ]] && [[ "$file" != *"RELEASE_NOTES"* ]] && [[ "$file" != *"ANNOUNCEMENT"* ]] && [[ "$file" != *"GITHUB_RELEASE"* ]] && [[ "$file" != *"DARK_MATTER"* ]]; then
    error "Version mismatch in $file: references ${CURRENT_VERSION}, current is $CURRENT_VERSION"
    VERSION_ERRORS=$((VERSION_ERRORS + 1))
  fi
  # Check for future version references (not current version) in filenames
  if [[ "$file" == *"v1.2.0"* ]] && [[ "$CURRENT_VERSION" != "1.2.0" ]] && [[ "$file" != *"coverage-strategy"* ]] && [[ "$file" != *"RELEASE_NOTES"* ]]; then
    error "Version mismatch in $file: references 1.2.0, current is $CURRENT_VERSION"
    VERSION_ERRORS=$((VERSION_ERRORS + 1))
  fi
  # Check for old version references (1.1.0, 1.0.0) in installation examples
  if grep -q "chicago-tdd-tools.*=.*\"1\.[01]\." "$file" 2>/dev/null; then
    if ! grep -q "chicago-tdd-tools.*=.*\"$CURRENT_VERSION\"" "$file" 2>/dev/null; then
      error "Outdated version in $file: should use $CURRENT_VERSION"
      VERSION_ERRORS=$((VERSION_ERRORS + 1))
    fi
  fi
done < <(find "$DOCS_DIR" -name "*.md" -type f 2>/dev/null)

if [ $VERSION_ERRORS -eq 0 ]; then
  echo "✅ Version numbers validated" >&2
else
  echo "❌ Found $VERSION_ERRORS version errors" >&2
fi
echo "" >&2

# Step 2: Validate build commands
echo "2️⃣  Checking build commands..." >&2
BUILD_ERRORS=0
while IFS= read -r file; do
  # Check for direct cargo commands in code blocks (should use cargo make)
  # Look for cargo test, cargo check, cargo build, cargo run, cargo publish in code blocks
  if grep -qE '```(bash|sh|shell|text|toml).*\ncargo (test|check|build|run|publish)' "$file" 2>/dev/null; then
    # Allow exceptions: cargo make, cargo doc, cargo audit, cargo outdated, cargo install
    # Also allow in comments explaining what NOT to do
    if ! grep -qE '(cargo (make|doc|audit|outdated|install)|should use|do not use|incorrect|wrong)' "$file" 2>/dev/null; then
      # Check if it's actually in a code example (not just mentioned)
      if grep -qE '```(bash|sh|shell).*\ncargo (test|check|build|run|publish)' "$file" 2>/dev/null; then
        error "Direct cargo command in $file: should use 'cargo make' instead"
        BUILD_ERRORS=$((BUILD_ERRORS + 1))
      fi
    fi
  fi
done < <(find "$DOCS_DIR" -name "*.md" -type f 2>/dev/null)

if [ $BUILD_ERRORS -eq 0 ]; then
  echo "✅ Build commands validated" >&2
else
  echo "❌ Found $BUILD_ERRORS build command errors" >&2
fi
echo "" >&2

# Step 3: Validate links (basic check - file existence)
echo "3️⃣  Checking links..." >&2
LINK_ERRORS=0
while IFS= read -r file; do
  # Extract markdown links [text](path)
  while IFS= read -r link; do
    # Skip external links (http/https)
    if [[ "$link" =~ ^https?:// ]]; then
      continue
    fi
    # Skip anchor links (#anchor)
    if [[ "$link" =~ ^# ]]; then
      continue
    fi
    # Remove anchor from path
    path="${link%%#*}"
    # Resolve relative path
    file_dir=$(dirname "$file")
    resolved_path="$file_dir/$path"
    # Check if file exists
    if [ ! -f "$resolved_path" ] && [ ! -d "$resolved_path" ]; then
      # Check if it's a relative path from docs root
      if [ ! -f "$DOCS_DIR/$path" ] && [ ! -d "$DOCS_DIR/$path" ]; then
        error "Broken link in $file: $link (resolved: $resolved_path)"
        LINK_ERRORS=$((LINK_ERRORS + 1))
      fi
    fi
  done < <(grep -oE '\[([^\]]+)\]\(([^)]+)\)' "$file" 2>/dev/null | sed -E 's/\[([^\]]+)\]\(([^)]+)\)/\2/')
done < <(find "$DOCS_DIR" -name "*.md" -type f 2>/dev/null)

if [ $LINK_ERRORS -eq 0 ]; then
  echo "✅ Links validated" >&2
else
  echo "❌ Found $LINK_ERRORS link errors" >&2
fi
echo "" >&2

# Step 4: Validate heading depth (max 4 levels)
echo "4️⃣  Checking heading depth..." >&2
HEADING_ERRORS=0
while IFS= read -r file; do
  # Check for headings deeper than 4 levels (#####)
  if grep -qE '^#####+' "$file" 2>/dev/null; then
    error "Heading too deep in $file: max 4 levels allowed"
    HEADING_ERRORS=$((HEADING_ERRORS + 1))
  fi
done < <(find "$DOCS_DIR" -name "*.md" -type f 2>/dev/null)

if [ $HEADING_ERRORS -eq 0 ]; then
  echo "✅ Heading depth validated" >&2
else
  echo "❌ Found $HEADING_ERRORS heading depth errors" >&2
fi
echo "" >&2

# Step 5: Validate required sections (Quick Navigation or See Also)
echo "5️⃣  Checking required sections..." >&2
SECTION_WARNINGS=0
while IFS= read -r file; do
  # Skip README files and process docs (they have different structure)
  if [[ "$file" == *"README.md" ]] || [[ "$file" == *"process/"* ]]; then
    continue
  fi
  # Check for Quick Navigation or See Also sections
  if ! grep -qiE '(quick navigation|see also|next steps)' "$file" 2>/dev/null; then
    warning "Missing cross-references in $file: should have 'Quick Navigation' or 'See Also' section"
    SECTION_WARNINGS=$((SECTION_WARNINGS + 1))
  fi
done < <(find "$DOCS_DIR/getting-started" "$DOCS_DIR/features" "$DOCS_DIR/reference" -name "*.md" -type f 2>/dev/null)

if [ $SECTION_WARNINGS -eq 0 ]; then
  echo "✅ Required sections validated" >&2
else
  echo "⚠️  Found $SECTION_WARNINGS section warnings" >&2
fi
echo "" >&2

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "📊 Validation Summary" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "Errors: $ERRORS" >&2
echo "Warnings: $WARNINGS" >&2
echo "" >&2

if [ $ERRORS -gt 0 ]; then
  echo "❌ Documentation validation failed" >&2
  exit 1
elif [ $WARNINGS -gt 0 ]; then
  echo "⚠️  Documentation validation passed with warnings" >&2
  exit 0
else
  echo "✅ All documentation checks passed" >&2
  exit 0
fi

