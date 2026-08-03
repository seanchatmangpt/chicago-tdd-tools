import os
import re

APP_GUIDE_DIR = "application-guide/src"
COOKBOOK_DIR = "cookbook/src"

def process_app_guide(file_path):
    with open(file_path, "r") as f:
        content = f.read()

    # Add content indicators if missing
    if "📚 **Reference** | 🔧 **How-to** | 🎓 **Tutorial**" not in content and "# " in content:
        # Some already have "> 🔧 How-to" or "> 💡 Explanation". Let's replace those or just append
        content = re.sub(r'(^# [^\n]+\n+)(?:> [^\n]+\n+)?', r'\1📚 **Reference** | 🔧 **How-to** | 🎓 **Tutorial**\n\n', content, count=1, flags=re.MULTILINE)

    # Fix fixtures.md API table
    if "fixtures.md" in file_path:
        content = content.replace(
            "| `capture_snapshot()` | `state: HashMap<String, String>` | `()` | Save test state snapshot |",
            "| `metadata_mut().capture_snapshot()` | `state: HashMap<String, String>` | `()` | Save state via metadata |"
        )
        content = content.replace(
            "| `snapshots()` | none | `&[HashMap<String, String>]` | Get all snapshots |",
            "| `metadata_ref().snapshots()` | none | `&[HashMap<String, String>]` | Get all snapshots via metadata |"
        )
        content = content.replace(
            "| `latest_snapshot()` | none | `Option<&HashMap<...>>` | Get most recent snapshot |",
            "| `with_scoped_metadata()` | `key`, `value` | `ScopedMetadata` | Scoped metadata |"
        )

    # Fix data-builders.md API table
    if "data-builders.md" in file_path:
        content = content.replace(
            "| `build_with_otel()` | `span_name: &str` | `(HashMap, Span)` | Build with OTEL span |",
            "| `build_with_otel()` | `span_name: &str` | `(HashMap, Span)` | GenericTestDataBuilder only |"
        )
        content = content.replace(
            "| `build_json()` | none | `Result<Value, Error>` | Build as JSON |",
            "| `build_json()` | none | `Result<Value, serde_json::Error>` | Build as JSON |"
        )
        content = content.replace(
            "| `build()` | none | `HashMap<String, String>` | Build as HashMap |",
            "| `try_build()` | none | `Result<HashMap<String, String>, String>` | Build with validation |"
        )

    with open(file_path, "w") as f:
        f.write(content)

def process_cookbook(file_path):
    with open(file_path, "r") as f:
        content = f.read()

    # Add Diataxis indicator
    if "> 🔧 **" not in content and "# " in content:
        # Extract title
        match = re.search(r'^# (.*)$', content, re.MULTILINE)
        if match:
            title = match.group(1).strip()
            header = f"> 🔧 **{title.upper()}** | **How-to** | Solve this problem\n"
            content = re.sub(r'(^# [^\n]+\n+)', f'\\1{header}\n', content, count=1, flags=re.MULTILINE)

            # Also add quick reference table if it doesn't have one
            if "| Aspect | Details |" not in content:
                table = """
## Quick Glance

| Aspect | Details |
|--------|---------|
| Problem | Addressed by this pattern |
| Solution | Core idea in 1-2 sentences |
| When To Use | Typical scenarios |
| When NOT To Use | Anti-patterns |
| Trade-offs | What you gain/lose |
| Complexity | Low/Medium/High |
| Real-World Example | Link to actual code |
"""
                # Insert before "## Context" or similar
                if "## Context" in content:
                    content = content.replace("## Context", f"{table}\n## Context")
                else:
                    content += f"\n{table}"

    with open(file_path, "w") as f:
        f.write(content)

for root, _, files in os.walk(APP_GUIDE_DIR):
    for file in files:
        if file.endswith(".md"):
            process_app_guide(os.path.join(root, file))

for root, _, files in os.walk(COOKBOOK_DIR):
    for file in files:
        if file.endswith(".md"):
            process_cookbook(os.path.join(root, file))

print("Updated docs successfully!")
