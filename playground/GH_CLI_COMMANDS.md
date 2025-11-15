# GitHub Actions CLI Commands

New commands added to the `playg` CLI for checking GitHub Actions status and workflows.

## Commands

### `playg gh stat`

Shows GitHub Actions status for all workflows in `.github/workflows/`.

**Usage:**
```bash
playg gh stat
```

**Output:**
Returns JSON with workflow statistics:
- `total_workflows`: Total number of workflow files found
- `valid_workflows`: Number of workflows with valid YAML syntax
- `invalid_workflows`: Number of workflows with invalid YAML syntax
- `workflows`: Array of workflow details (name, path, jobs count, validity)

**Example:**
```bash
$ playg gh stat
{"invalid_workflows":0,"total_workflows":5,"valid_workflows":5,"workflows":[...]}
```

### `playg gh list`

Lists all GitHub Actions workflows.

**Usage:**
```bash
playg gh list [--format FORMAT]
```

**Options:**
- `--format names` - Show workflow names (default)
- `--format paths` - Show workflow file paths
- `--format json` - Show full JSON output

**Example:**
```bash
$ playg gh list
📋 GitHub Actions Workflows:
  • ci
  • release
  • benchmark
  • docs
  • stale
```

### `playg gh check`

Validates all GitHub Actions workflows for:
- YAML syntax errors
- Missing explicit permissions (security best practice)
- Deprecated actions usage
- Other best practice violations

**Usage:**
```bash
playg gh check [--fix] [--verbose LEVEL]
```

**Options:**
- `--fix` - Attempt to auto-fix issues (not yet implemented)
- `--verbose N` - Verbosity level (0-3)

**Example:**
```bash
$ playg gh check
🔍 Validating GitHub Actions workflows...

✅ All workflows validated successfully!
```

### `playg gh runs`

Shows recent workflow runs. Requires `gh` CLI to be installed and authenticated.

**Usage:**
```bash
playg gh runs [--limit N] [--workflow NAME]
```

**Options:**
- `--limit N` - Number of runs to fetch (default: 10)
- `--workflow NAME` - Filter by workflow name

**Example:**
```bash
$ playg gh runs --limit 5
🔄 Fetching recent workflow runs...

STATUS    WORKFLOW  COMMIT        CREATED_AT
✅ success  CI         feat: ...    2025-11-14T...
```

### `playg gh open`

Opens the GitHub Actions page for the current repository in your browser.

**Usage:**
```bash
playg gh open
```

**Example:**
```bash
$ playg gh open
🌐 Opening GitHub Actions...
Actions URL: https://github.com/user/repo/actions
```

## Implementation Details

### File Structure

- **Location**: `playground/src/cli/gh.rs`
- **Pattern**: Uses `clap-noun-verb` pattern with `#[verb]` macros
- **Dependencies**: Added `serde_yaml` to `playground/Cargo.toml`

### Key Features

1. **Workflow Discovery**: Automatically scans `.github/workflows/` directory
2. **YAML Validation**: Validates syntax using `serde_yaml`
3. **Security Checks**: Identifies missing permissions and deprecated actions
4. **GitHub CLI Integration**: Integrates with `gh` CLI for live status
5. **JSON Output**: Returns structured data for programmatic use

### Validation Rules

The `check` command validates:

1. **YAML Syntax**: Ensures all workflow files are valid YAML
2. **Security**: Checks for explicit `permissions:` declarations
3. **Deprecated Actions**:
   - `actions/create-release@v1` → Use `softprops/action-gh-release@v2`
   - `actions/upload-release-asset@v1` → Use artifact upload pattern
   - `actions/cache@v1` / `v2` → Use `Swatinem/rust-cache@v2` for Rust

### Error Handling

All commands use `clap_noun_verb::Result` for consistent error handling. Commands fail gracefully and provide helpful error messages.

## Testing

To test the commands:

```bash
# From the playground directory
cargo build --bin playg

# Test GitHub Actions status
cargo run --bin playg -- gh stat

# Test workflow validation
cargo run --bin playg -- gh check

# Test workflow listing
cargo run --bin playg -- gh list

# Open GitHub Actions (requires gh CLI or git remote)
cargo run --bin playg -- gh open
```

## Example Session

```bash
$ cd playground

# Check status
$ cargo run --quiet --bin playg -- gh stat
{"invalid_workflows":0,"total_workflows":5,"valid_workflows":5,...}

# Validate workflows
$ cargo run --quiet --bin playg -- gh check
🔍 Validating GitHub Actions workflows...
✅ All workflows validated successfully!

# List workflows
$ cargo run --quiet --bin playg -- gh list
📋 GitHub Actions Workflows:
  • ci
  • release
  • benchmark
  • docs
  • stale

# Open in browser
$ cargo run --quiet --bin playg -- gh open
🌐 Opening GitHub Actions...
Actions URL: https://github.com/seanchatmangpt/chicago-tdd-tools/actions
```

## Future Enhancements

Potential improvements:
1. Auto-fix for common issues (`--fix` flag implementation)
2. Workflow run history visualization
3. Matrix job status breakdown
4. Workflow duration analytics
5. Cost estimation for workflow runs
6. Integration with GitHub API for more detailed information

## Notes

- Commands work offline except for `gh runs` (requires network and `gh` CLI)
- YAML validation is syntactic only, not semantic
- Security checks are recommendations, not requirements
- The `check` command helps maintain best practices

## Dependencies

Added to `playground/Cargo.toml`:
```toml
serde_yaml = "^0.9"
```

Existing dependencies used:
- `clap-noun-verb` - CLI framework
- `clap-noun-verb-macros` - Verb macros
- `serde` - JSON serialization
- `std::process::Command` - External command execution
