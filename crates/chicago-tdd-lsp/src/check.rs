//! Core diagnostic logic — shared between the minimal and lsp-max-runtime servers.

/// A violation found in a `Cargo.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// 0-indexed line number.
    pub line: u32,
    /// 0-indexed start column (byte offset of the key).
    pub char_start: u32,
    /// 0-indexed end column.
    pub char_end: u32,
    /// The diagnostic code string.
    pub code: &'static str,
    /// Human-readable message.
    pub message: &'static str,
    /// The law text shown on hover — the canonical reason this is forbidden.
    pub law_text: &'static str,
    /// Canonical fix guidance for code actions / hover detail.
    pub fix_guidance: &'static str,
}

const CODE: &str = "CTDD-DEV-001";
const MESSAGE: &str = "`chicago-tdd-tools` must only appear in [dev-dependencies], \
    not in [dependencies]. It is a test-only library and must \
    never be distributed as part of a published crate.";
const LAW: &str = "Chicago TDD Law §DEV-001: Test-only libraries must not appear \
    in [dependencies]. A crate that pulls chicago-tdd-tools into its published \
    dependency graph contaminates downstream consumers with test infrastructure.";
const FIX: &str = "Move `chicago-tdd-tools` from [dependencies] to [dev-dependencies].";

/// Scan `Cargo.toml` text and return all violations.
pub fn find_violations(text: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut in_bad_section = false;

    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') {
            let section = trimmed.trim_matches(|c| c == '[' || c == ']').to_lowercase();
            let is_dev = section == "dev-dependencies" || section.starts_with("dev-dependencies.");
            in_bad_section =
                !is_dev && (section == "dependencies" || section.starts_with("dependencies."));
            continue;
        }

        if in_bad_section && is_chicago_tdd_entry(trimmed) {
            let char_start = line.find("chicago-tdd-tools").unwrap_or(0) as u32;
            let char_end = char_start + "chicago-tdd-tools".len() as u32;
            violations.push(Violation {
                line: idx as u32,
                char_start,
                char_end,
                code: CODE,
                message: MESSAGE,
                law_text: LAW,
                fix_guidance: FIX,
            });
        }
    }

    violations
}

fn is_chicago_tdd_entry(line: &str) -> bool {
    let key = line.split('=').next().unwrap_or("").trim();
    key == "chicago-tdd-tools" || key == "chicago_tdd_tools"
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::find_violations;

    #[test]
    fn no_violation_when_dev_dependency() {
        let toml = "[dev-dependencies]\nchicago-tdd-tools = { path = \"..\" }\n";
        assert!(find_violations(toml).is_empty());
    }

    #[test]
    fn violation_when_in_dependencies() {
        let toml = "[dependencies]\nchicago-tdd-tools = { path = \"..\" }\n";
        let v = find_violations(toml);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "CTDD-DEV-001");
        assert_eq!(v[0].line, 1);
    }

    #[test]
    fn no_violation_for_unrelated_deps() {
        let toml = "[dependencies]\nserde = \"1\"\ntokio = \"1\"\n";
        assert!(find_violations(toml).is_empty());
    }

    #[test]
    fn only_dependencies_section_flagged_not_dev() {
        let toml = "[dependencies]\nchicago-tdd-tools = \"26.6.12\"\n\
                    [dev-dependencies]\nchicago-tdd-tools = \"26.6.12\"\n";
        let v = find_violations(toml);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].line, 1);
    }

    #[test]
    fn underscore_variant_also_caught() {
        let toml = "[dependencies]\nchicago_tdd_tools = { path = \"..\" }\n";
        let v = find_violations(toml);
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn violation_carries_law_text_and_fix() {
        let toml = "[dependencies]\nchicago-tdd-tools = \"26.6.12\"\n";
        let v = find_violations(toml);
        assert!(!v[0].law_text.is_empty());
        assert!(!v[0].fix_guidance.is_empty());
        assert!(v[0].fix_guidance.contains("dev-dependencies"));
    }
}
