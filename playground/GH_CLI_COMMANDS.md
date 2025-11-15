# GitHub Actions CLI Commands

Reference implementation demonstrating **clap-noun-verb 3.7.1** best practices through practical GitHub Actions management commands. These commands showcase:

- **Attribute Macro Pattern**: Zero-boilerplate `#[verb]` macros
- **Type Inference**: Arguments inferred from Rust function signatures
- **JSON Output**: Automatic serialization for automation
- **Multiple Output Formats**: JSON, YAML, TOML, Table, TSV
- **Composability**: Building complex CLIs from simple functions

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
- **Pattern**: Uses **clap-noun-verb 3.7.1** with `#[verb]` macros for automatic command registration
- **Dependencies**: `serde_yaml` for YAML validation

### How clap-noun-verb Powers the Commands

#### 1. Type Inference from Function Signatures

The playground's GitHub commands demonstrate clap-noun-verb's **automatic type inference**:

```rust
// From playground/src/cli/gh.rs

/// Show GitHub Actions status
#[verb]
fn stat(verbose: usize) -> Result<GhStatus> {
    // verbose: usize → automatically becomes -v/-vv/-vvv count flag
    // Return type: GhStatus → automatically serialized to JSON
}

/// List all GitHub Actions workflows
#[verb]
fn list(format: Option<String>) -> Result<Vec<String>> {
    // format: Option<String> → becomes optional --format <value> argument
    // Result<T> → consistent error handling
}

/// Validate GitHub Actions workflows
#[verb]
fn check(fix: bool, verbose: usize) -> Result<Vec<String>> {
    // fix: bool → becomes --fix flag
    // verbose: usize → becomes -v/-vv/-vvv count flag
}
```

#### 2. Auto-Discovery via linkme

All verb handlers are automatically registered at compile time:

```rust
// In playground/src/cli/mod.rs
pub mod gh;  // ← linkme automatically discovers all #[verb] functions here!

// No enum boilerplate needed!
// No match statements needed!
// Just import the module and clap-noun-verb handles the rest.
```

#### 3. Automatic Serialization

All return types are automatically serialized to JSON:

```rust
#[derive(Serialize)]  // ← Required for clap-noun-verb
struct GhStatus {
    workflows: Vec<WorkflowStatus>,
    total_workflows: usize,
    valid_workflows: usize,
    invalid_workflows: usize,
}

// Automatically serialized by clap-noun-verb:
// $ playg gh stat
// {"workflows":[...],"total_workflows":5,...}
```

#### 4. Multiple Output Format Support

clap-noun-verb provides automatic format conversion:

```bash
# JSON (default - from Serialize derive)
$ playg gh stat --format json
{"workflows":[...],"total_workflows":5}

# YAML
$ playg gh stat --format yaml
workflows:
  - name: ci
    ...

# TOML
$ playg gh stat --format toml
total_workflows = 5
...

# Table
$ playg gh stat --format table
| name | path           | jobs | valid |
|------|----------------|------|-------|
| ci   | .github/...    | 8    | true  |

# TSV (Tab-Separated Values)
$ playg gh stat --format tsv
name	path	jobs	valid
ci	.github/workflows/ci.yml	8	true
```

### Key Features

1. **Workflow Discovery**: Automatically scans `.github/workflows/` directory
2. **YAML Validation**: Validates syntax using `serde_yaml`
3. **Security Checks**: Identifies missing permissions and deprecated actions
4. **GitHub CLI Integration**: Integrates with `gh` CLI for live status
5. **JSON Output**: Returns structured data for programmatic use
6. **Zero Boilerplate**: No argument parsing code needed—inferred from function signatures

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

## Why clap-noun-verb?

### Before (Traditional clap)
```rust
// Lots of boilerplate!
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Gh {
        #[command(subcommand)]
        command: GhCommands,
    },
}

#[derive(Subcommand)]
enum GhCommands {
    Stat { #[arg(short = 'v', action = "count")] verbose: usize },
    List { #[arg(long)] format: Option<String> },
    Check { #[arg(long)] fix: bool, #[arg(short = 'v', action = "count")] verbose: usize },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Gh { command } => match command {
            GhCommands::Stat { verbose } => { /* ... */ }
            GhCommands::List { format } => { /* ... */ }
            // ... more match arms
        }
    }
}
```

### After (clap-noun-verb)
```rust
// Clean, focused functions!
#[verb]
fn stat(verbose: usize) -> Result<GhStatus> { /* ... */ }

#[verb]
fn list(format: Option<String>) -> Result<Vec<String>> { /* ... */ }

#[verb]
fn check(fix: bool, verbose: usize) -> Result<Vec<String>> { /* ... */ }

fn main() -> Result<()> {
    clap_noun_verb::run()  // That's it!
}
```

### Benefits
- **75% less code**: No enums, match statements, or manual argument parsing
- **Type-safe**: Compiler enforces correct argument types
- **Easy to extend**: Add a new verb? Just add a function!
- **Automatic serialization**: Return types automatically become JSON output
- **Multiple formats**: Free support for YAML, TOML, Table, TSV
- **Perfect for agents**: JSON output makes it ideal for AI/agent integration

## Notes

- Commands work offline except for `gh runs` (requires network and `gh` CLI)
- YAML validation is syntactic only, not semantic
- Security checks are recommendations, not requirements
- The `check` command helps maintain best practices
- All commands use clap-noun-verb's `Result<T>` for consistent error handling

## Dependencies

Added to `playground/Cargo.toml`:
```toml
serde_yaml = "^0.9"
serde_json = "^1.0"
```

From clap-noun-verb:
- `clap` - Argument parsing (transitive)
- `clap-noun-verb` - Framework
- `clap-noun-verb-macros` - Attribute macros
- `linkme` - Auto-discovery (transitive)
- `serde` - Serialization framework
- `serde_json` - JSON serialization

## Further Reading

- See `CLI_ARCHITECTURE.md` for deep dive into the clap-noun-verb setup
- See `EXTENDING_CLI.md` for guides on adding new commands
- See the main `README.md` for complete playground overview
- Official clap-noun-verb docs: https://docs.rs/clap-noun-verb/
