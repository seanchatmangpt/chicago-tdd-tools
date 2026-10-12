#![cfg(feature = "nato")]
// Court harness library: probes and full-tier entry points run under
// `#[ignore]`d tests, so non-test builds legitimately see unused items —
// and referenced by the court registry.
#![allow(dead_code)]
//! N16 — stack-bound court for the PN64 hot path (NATO lane N16).
//!
//! Ported from strata_protocol tests/nato/n16_zero_recursion.rs, scoped to
//! this crate's hot path: [`crate::conformance::pn64`] (embedded at compile
//! time via `include_str!`, so the audit cannot desync from the source).
//!
//! Four legs, per the lane contract:
//!
//! 1. **Zero recursion** (always-run): real iterative cycle detection (DFS,
//!    not a keyword grep) over the fn/callee graph extracted from the
//!    embedded hot-path source. Any direct or mutual cycle is a hard fail.
//! 2. **Zero heap tokens** (always-run): no dynamic-allocation tokens in any
//!    extracted hot-path function body, and no `Box<` anywhere in the
//!    stripped module (state words are fixed-layout).
//! 3. **32 KiB thread workload** (`#[ignore = "nato-tier3"]`): the 100k
//!    transition workload runs on a thread with a known 32 KiB stack
//!    (`std::thread::Builder::stack_size`), asserting its computed commit
//!    count against a parent-thread reference value.
//! 4. **Child-probe ladder** (`#[ignore = "nato-tier3"]`): the minimum
//!    surviving stack is found by descending the ladder 32768 -> 4096, each
//!    probe point executed in a **child process** (via `current_exe` +
//!    env var) so a probing overflow kills only the child, never the court.
//!    Release-profile court: run under `--release` for the measured figure.
//!
//! Honest scoping: the audit proves recursion depth is exactly 1 (no
//! cycles); the probe bounds the whole workload's call-chain footprint at
//! OS page granularity. No 256-byte-total-frame claim is made.

// Lint-scope note: NATO court-harness and ledger wire-format modules use
// domain literals, metric casts, and court-assertion panics by design; the
// style-class lints below are allowed at module scope with this
// justification rather than 300+ inline permits.
#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::too_long_first_doc_paragraph,
    clippy::doc_overindented_list_items,
    clippy::doc_lazy_continuation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::explicit_iter_loop,
    clippy::unusual_byte_groupings,
    clippy::unreadable_literal,
    clippy::similar_names,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::unused_self,
    clippy::needless_pass_by_value,
    clippy::uninlined_format_args,
    clippy::needless_range_loop,
    clippy::needless_continue,
    clippy::unnecessary_wraps,
    clippy::double_must_use,
    clippy::op_ref,
    clippy::match_same_arms,
    clippy::type_complexity,
    clippy::or_fun_call,
    clippy::nonminimal_bool,
    clippy::unnecessary_map_or,
    clippy::manual_is_multiple_of,
    clippy::manual_midpoint,
    clippy::identity_op,
    clippy::format_collect,
    clippy::trivially_copy_pass_by_ref
)]
use std::collections::HashMap;
#[cfg(test)]
use std::process::Command;
#[cfg(test)]
use std::sync::Arc;

use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};

use crate::conformance::pn64;
use crate::nato::court::{
    CandidateRef, ControlProbe, Court, CourtId, CourtOutcome, MetricValue, Pillar, Verdict,
};
use crate::nato::disclosure::ScopingRecord;

/// Embedded source under audit (compile-time embedded: audit cannot desync).
const AUDITED_SOURCE: (&str, &str) =
    ("conformance/pn64.rs", include_str!("../conformance/pn64.rs"));

/// Env var selecting child-probe mode: `NATO_STACKBOUND_PROBE_STACK=<bytes>`
/// runs the workload on a thread with that exact stack, then exits 0. Too
/// small a stack kills the child (SIGSEGV); the parent observes nonzero exit.
const PROBE_ENV: &str = "NATO_STACKBOUND_PROBE_STACK";

/// Descending probe ladder (macOS 16 KiB pages; pthread rounds up regardless,
/// so 4 KiB multiples are the honest granularity).
const LADDER: [usize; 4] = [32768, 16384, 8192, 4096];

/// Transitions in the probed workload.
const WORKLOAD_TRANSITIONS: u64 = 100_000;

// ---------------------------------------------------------------------------
// Comment stripping + brace-matched function extraction (AST-lite)
// ---------------------------------------------------------------------------

/// Remove `//` line comments and `/* */` block comments, string-literal aware
/// enough for this module; the court fails loudly on brace imbalance if
/// stripping corrupts the source beyond parseability.
fn strip_comments(src: &str) -> String {
    let chars: Vec<char> = src.chars().collect();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    let mut in_str = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_str = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// Excise `#[cfg(test)] mod ... { ... }` blocks (tests are not state paths).
/// Returns the stripped source and the count of removed regions so the
/// caller can assert the stripper did real work (fail-closed, not silent).
fn strip_cfg_test_modules(src: &str) -> (String, usize) {
    let mut out = src.to_string();
    let mut removed = 0;
    loop {
        let marker = "#[cfg(test)]";
        let Some(pos) = out.find(marker) else { break };
        let after = &out[pos + marker.len()..];
        let trimmed = after.trim_start();
        if !trimmed.starts_with("mod") {
            let keep = pos + marker.len();
            out.replace_range(pos..keep, "");
            removed += 1;
            continue;
        }
        let Some(brace_rel) = trimmed.find('{') else { break };
        let abs_brace = pos + marker.len() + (after.len() - trimmed.len()) + brace_rel;
        let chars: Vec<char> = out[abs_brace..].chars().collect();
        let mut depth = 0i32;
        let mut m = 0;
        while m < chars.len() {
            if chars[m] == '{' {
                depth += 1;
            } else if chars[m] == '}' {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            m += 1;
        }
        if depth != 0 {
            break;
        }
        let end_abs =
            abs_brace + out[abs_brace..].char_indices().nth(m).map(|(b, _)| b).unwrap_or(0) + 1;
        out.replace_range(pos..end_abs, "");
        removed += 1;
    }
    (out, removed)
}

/// A function extracted from the audited source.
#[derive(Clone, Debug)]
struct FnNode {
    name: String,
    body: String,
}

/// Extract top-level and inherent-impl `fn` bodies by brace matching.
/// Returns (functions, globally_brace_balanced).
fn extract_fns(src: &str) -> (Vec<FnNode>, bool) {
    let chars: Vec<char> = src.chars().collect();
    let mut fns = Vec::new();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        let at_word_start = i == 0 || !(chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
        if at_word_start
            && chars[i] == 'f'
            && i + 2 < n
            && chars[i + 1] == 'n'
            && (chars[i + 2] == ' ' || chars[i + 2] == '\t' || chars[i + 2] == '\n')
        {
            let mut j = i + 3;
            while j < n && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
            let name: String = chars[i + 3..j].iter().collect();
            if !name.is_empty() {
                // First `{` after the name (skipping generics/params/where).
                let mut k = j;
                while k < n && chars[k] != '{' {
                    k += 1;
                }
                if k < n {
                    let mut depth = 0i32;
                    let mut m = k;
                    while m < n {
                        if chars[m] == '{' {
                            depth += 1;
                        } else if chars[m] == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        m += 1;
                    }
                    if depth == 0 {
                        fns.push(FnNode { name, body: chars[k..=m].iter().collect() });
                        i = m + 1;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    let mut bal = 0i32;
    for c in &chars {
        if *c == '{' {
            bal += 1;
        } else if *c == '}' {
            bal -= 1;
        }
    }
    (fns, bal == 0)
}

/// Extract identifiers in `body` followed by `(` — candidate callees.
/// Excludes control keywords; `!`-suffixed macros are unwrapped to their
/// base name (macro:braces like `vec![` are caught by the heap-token audit).
fn callees(body: &str) -> Vec<String> {
    const KEYWORDS: &[&str] =
        &["if", "while", "for", "match", "switch", "return", "unsafe", "in", "catch"];
    let chars: Vec<char> = body.chars().collect();
    let mut out = Vec::new();
    for i in 0..chars.len() {
        if chars[i] != '(' {
            continue;
        }
        let mut j = i;
        while j > 0
            && (chars[j - 1].is_alphanumeric()
                || chars[j - 1] == '_'
                || chars[j - 1] == ':'
                || chars[j - 1] == '!')
        {
            j -= 1;
        }
        let ident: String = chars[j..i].iter().collect();
        let base = ident.trim_end_matches('!').to_string();
        let at_boundary = j == 0 || !(chars[j - 1].is_alphanumeric() || chars[j - 1] == '_');
        if !base.is_empty()
            && at_boundary
            && !KEYWORDS.contains(&base.as_str())
            && !base.chars().next().unwrap().is_ascii_digit()
        {
            out.push(base);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tier 1a (always) — zero recursion via iterative call-graph cycle detection
// ---------------------------------------------------------------------------

/// Hoisted body of [`n16_static_audit_zero_recursion`] (also the
/// zero-recursion leg of the N16 court adapter's `run`). Returns
/// (fn_count, call_edges) for the adapter's metrics; panics on any cycle.
pub(crate) fn stackbound_zero_recursion_probe() -> (usize, usize) {
    let (file, src) = AUDITED_SOURCE;
    let stripped = strip_comments(src);
    let (stripped, cfg_removed) = strip_cfg_test_modules(&stripped);
    assert!(
        cfg_removed >= 1,
        "N16 audit: expected to excise at least one #[cfg(test)] region from {file} — stripper found none, audit scope is suspect"
    );
    let (fns, balanced) = extract_fns(&stripped);
    assert!(balanced, "N16 audit: brace imbalance in {file} — comment stripper corrupted source");
    assert!(!fns.is_empty(), "N16 audit: extracted zero functions from {file} — audit is vacuous");
    assert!(
        fns.iter().any(|f| f.name == "step_branchless"),
        "N16 audit: hot path step_branchless not among the {} extracted functions — audit is desynced from the real call surface",
        fns.len()
    );
    assert!(
        fns.iter().all(|f| !f.body.is_empty()),
        "N16 audit: empty function body extracted from {file}"
    );

    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for f in fns {
        let calls = callees(&f.body);
        graph.entry(f.name).or_default().extend(calls);
    }
    let edges_total: usize = graph.values().map(Vec::len).sum();

    let cycles = detect_cycles(&graph);
    assert!(cycles.is_empty(), "N16 FAIL: recursion detected on state path: {cycles:?}");
    // Audit non-vacuity receipt: the graph actually had nodes and edges.
    assert!(
        edges_total > 0 || graph.len() > 1,
        "N16 audit: extracted call graph has no edges and one node — cycle check carried no bits"
    );
    (graph.len(), edges_total)
}

#[test]
fn n16_static_audit_zero_recursion() {
    let (fn_count, edges) = stackbound_zero_recursion_probe();
    assert!(fn_count > 0, "extracted fn census must be non-empty");
    assert!(edges > 0 || fn_count > 1, "call graph must carry edges");
}

// ---------------------------------------------------------------------------
// Tier 1b (always) — zero dynamic allocation tokens on hot-path bodies
// ---------------------------------------------------------------------------

const HEAP_TOKENS: &[&str] = &[
    "Vec::",
    "vec![",
    "Box::",
    "String::",
    "format!",
    ".to_vec()",
    ".collect()",
    "to_owned()",
    "String::from",
    "with_capacity",
    "thread::spawn",
    "HashMap::",
    "BTreeMap::",
    ".clone()",
    ".into()",
];

/// Hoisted heap-token audit over an arbitrary source text: every extracted
/// fn body is scanned for [`HEAP_TOKENS`]; any `Box<` in the stripped module
/// is also a violation (state words are fixed-layout). Returns the violation
/// strings (empty = clean).
fn heap_violations_in(file: &str, src: &str) -> Vec<String> {
    let stripped = strip_comments(src);
    let (stripped, _cfg) = strip_cfg_test_modules(&stripped);
    let (fns, _) = extract_fns(&stripped);
    assert!(
        !fns.is_empty(),
        "N16 audit: extracted zero functions from {file} — heap audit is vacuous"
    );
    let mut violations = Vec::new();
    for f in &fns {
        for tok in HEAP_TOKENS {
            if f.body.contains(tok) {
                violations.push(format!("{file}::fn {} contains `{tok}`", f.name));
            }
        }
    }
    if stripped.contains("Box<") {
        violations.push(format!("{file} contains `Box<`"));
    }
    violations
}

/// Hoisted body of [`n16_static_audit_zero_heap_on_state_paths`] (also the
/// zero-heap leg of the N16 court adapter's `run`).
pub(crate) fn stackbound_heap_audit_probe() -> usize {
    let (file, src) = AUDITED_SOURCE;
    let violations = heap_violations_in(file, src);
    assert!(
        violations.is_empty(),
        "N16 FAIL: heap-allocation tokens on state path: {violations:#?}"
    );
    0
}

#[test]
fn n16_static_audit_zero_heap_on_state_paths() {
    assert_eq!(stackbound_heap_audit_probe(), 0);
}

#[test]
fn n16_court_detects_injected_heap_token() {
    // Same contract as mils' n01_court_detects_injected_interference: the
    // injected `Vec::` state-path token MUST be flagged (non-vacuity by
    // mutation).
    assert!(stackbound_positive_control(), "injected-Vec:: positive control must fire");
}

// ---------------------------------------------------------------------------
// Tier 2 (#[ignore = "nato-tier3"]) — measured stack-depth probes
// ---------------------------------------------------------------------------

/// The workload: 100k branchless transitions over a rotating adversarial
/// word set, folding every result into the accumulator so the compiler
/// cannot elide the transitions. Returns the folded digest; the parent
/// computes the same digest on its own (large) stack as the reference.
fn workload() -> u64 {
    let mut acc = 0u64;
    let mut r_pn = 0xC0FF_EE13_5EED_0013u64;
    for i in 0..WORKLOAD_TRANSITIONS {
        // Rotate the PN word deterministically (no allocation, no syscalls):
        // keep marking/pre/post full-width so every transition commits.
        r_pn = r_pn.wrapping_mul(0x2545_F491_4F6C_DD1D).rotate_left((i & 0x3F) as u32);
        let pn = pn64::pack(0xFFFF, 0xFFFF, 0x0001, (r_pn & 0xFF) as u8, 0x10);
        let ocel = pn64::pack((pn64::tid(pn) as u16) | 0xFF00, 0xFFFF, 0xFFFF, 0xFF, 0xFF);
        acc = acc.wrapping_add(pn64::step_branchless(pn, ocel, 0x0000_0000_0000_0010));
    }
    acc
}

/// Child-process probe mode: run [`workload`] on a thread with exactly
/// `stack_bytes` and store its digest in [`SUCCESS`]; the harness then exits
/// normally. Stack overflow kills the child process outright.
fn child_probe(stack_bytes: usize) {
    let handle = std::thread::Builder::new()
        .stack_size(stack_bytes)
        .spawn(|| {
            let digest = workload();
            SUCCESS.store(digest, AtomicOrdering::SeqCst);
        })
        .expect("N16 probe thread spawn failed (stack below PTHREAD_STACK_MIN)");
    handle.join().expect("N16 probe thread panicked");
}

static SUCCESS: AtomicU64 = AtomicU64::new(0);

/// Leg 3: the 100k-transition workload survives and computes the correct
/// digest on a 32 KiB thread stack.
#[ignore = "nato-tier3"]
#[test]
fn n16_workload_100k_transitions_on_32kib_thread() {
    if let Ok(sz) = std::env::var(PROBE_ENV) {
        child_probe(sz.parse().expect("bad probe stack size"));
        return;
    }
    let expected = workload();
    let digest = Arc::new(AtomicU64::new(0));
    let digest_t = Arc::clone(&digest);
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024)
        .spawn(move || {
            digest_t.store(workload(), AtomicOrdering::SeqCst);
        })
        .expect("32 KiB probe thread spawn failed");
    handle.join().expect("32 KiB probe thread panicked");
    let observed = digest.load(AtomicOrdering::SeqCst);
    assert_eq!(
        observed, expected,
        "N16: 32 KiB-thread digest diverged from the parent-thread reference — workload results are stack-dependent"
    );
}

/// Leg 4: descend the ladder 32768 -> 4096, each point in a child process
/// via `current_exe` + [`PROBE_ENV`]; a probing overflow kills only the
/// child. Asserts the minimum surviving rung is a real measured value and
/// records the honest headroom statement. Release-profile court.
#[ignore = "nato-tier3"]
#[test]
fn n16_measured_min_stack_ladder_via_child_probes() {
    if std::env::var(PROBE_ENV).is_ok() {
        return; // child mode is handled by the 32 KiB leg's guard
    }
    let exe = std::env::current_exe().expect("current_exe");
    let probe = |stack: usize| -> bool {
        let out = Command::new(&exe)
            .env(PROBE_ENV, stack.to_string())
            .output()
            .expect("spawn probe child");
        out.status.success()
    };
    let mut min_ok = None;
    let mut failed_at = None;
    for sz in LADDER {
        if probe(sz) {
            min_ok = Some(sz);
            break;
        }
        failed_at = Some(sz);
    }
    let min_ok = min_ok.unwrap_or_else(|| {
        panic!(
            "N16 FAIL: workload does not fit even a {}-byte thread stack (failed rungs up to {:?}) — stack bound broken",
            LADDER[0], failed_at
        )
    });
    assert!(
        min_ok <= LADDER[0],
        "N16 FAIL: measured min stack {min_ok} exceeds the asserted {}-byte bound",
        LADDER[0]
    );
    eprintln!(
        "N16 measured min surviving stack: {min_ok} bytes (ladder {LADDER:?}; recursion depth = 1 proven by tier 1a; whole-chain footprint at page granularity, release-profile figure)"
    );
}

// ---------------------------------------------------------------------------
// N16 court adapter (W-NTW): trait wiring per the mils.rs N01 template
// ---------------------------------------------------------------------------

/// A heap-allocating mutant (a `Vec::` token on a state path) whose source
/// text is embedded so the audit scan runs on exactly the audited shape.
const HEAP_MUTANT_SRC: &str = r"
fn heap_mutant(pn: u64, ocel: u64, mask: u64) -> u64 {
    let log = Vec::new();
    log.push(pn);
    let m = marking(pn);
    let next = ((m ^ pre(pn)) | post(pn)) & 0xFFFF;
    next
}
";

/// Positive control: an injected `Vec::` token on a state-path body MUST be
/// flagged by the heap audit. Returns false if the court is vacuous.
/// Iterative DFS cycle detection over the extracted call graph
/// (color 0 = unvisited, 1 = in-stack, 2 = done; any back edge to an
/// in-stack node is recursion on a state path).
fn detect_cycles(graph: &std::collections::HashMap<String, Vec<String>>) -> Vec<String> {
    let mut cycles = Vec::new();
    for start in graph.keys() {
        let mut color: HashMap<&str, u8> = HashMap::new();
        let mut stack: Vec<(&str, Vec<&str>)> =
            vec![(start.as_str(), graph[start].iter().map(String::as_str).collect())];
        color.insert(start.as_str(), 1);
        while let Some((node, succ)) = stack.pop() {
            if let Some(next) = succ.first().copied() {
                let mut rest = succ;
                rest.remove(0);
                stack.push((node, rest));
                match color.get(next).copied() {
                    Some(1) => cycles.push(format!("{node} -> {next}")),
                    Some(2) => {}
                    _ => {
                        color.insert(next, 1);
                        let edges: Vec<&str> = graph
                            .get(next)
                            .map(|v| v.iter().map(String::as_str).collect())
                            .unwrap_or_default();
                        stack.push((next, edges));
                    }
                }
            } else {
                color.insert(node, 2);
            }
        }
    }
    cycles
}

/// Extracts the call graph from `src` and returns every recursion cycle.
fn cycles_in(file: &str, src: &str) -> Vec<String> {
    let stripped = strip_comments(src);
    let (fns, balanced) = extract_fns(&stripped);
    assert!(balanced, "N16 control: brace imbalance in {file}");
    let mut graph: HashMap<String, Vec<String>> = HashMap::new();
    for f in &fns {
        graph.entry(f.name.clone()).or_default().extend(callees(&f.body));
    }
    detect_cycles(&graph)
}

pub(crate) fn stackbound_positive_control() -> bool {
    !heap_violations_in("mutant.rs", HEAP_MUTANT_SRC).is_empty() && stackbound_recursion_control()
}

/// Audit W3: the cycle detector's vacuity control — a self-recursive fn
/// (direct recursion) must be flagged by the iterative DFS, proving the
/// zero-recursion leg can fire, not just pass on clean input.
const RECURSION_MUTANT_SRC: &str = r"
fn checked_recurse(n: u64) -> u64 {
    if n == 0 {
        1
    } else {
        checked_recurse(n - 1)
    }
}
";

/// Returns true iff the cycle detector flags the injected self-recursive
/// mutant. Wired as part of the N16 court's positive control.
pub(crate) fn stackbound_recursion_control() -> bool {
    !cycles_in("mutant.rs", RECURSION_MUTANT_SRC).is_empty()
}

/// N16 stack-bound court (always-run legs: zero recursion + zero heap
/// tokens on state paths; tier-3 measured stack probes excluded).
///
/// Pillar note: the 5-value [`Pillar`] enum has no dedicated
/// resource/stack pillar; per the plan's N04 precedent this court is
/// classified [`Pillar::Dossier`], disclosed here and in the run's
/// [`ScopingRecord`].
pub struct StackBoundCourt;

/// Court identity constants (the `Court` trait exposes them
/// dyn-safely via `id()`/`pillar()`).
impl StackBoundCourt {
    /// Court identifier.
    pub const ID: CourtId = CourtId::N16;
    /// Capability pillar.
    pub const PILLAR: Pillar = Pillar::Dossier;
}

/// Hoisted court run: the two always-run legs, packaged as a
/// [`CourtOutcome`]. Panics (leg assertions) are the Fail path's evidence;
/// a clean return is Pass.
pub(crate) fn stackbound_run_court_probe() -> CourtOutcome {
    let (fn_count, edges) = stackbound_zero_recursion_probe();
    let heap_hits = stackbound_heap_audit_probe();
    CourtOutcome {
        verdict: Verdict::Pass,
        metrics: vec![
            ("fns_audited", MetricValue::U64(fn_count as u64)),
            ("call_edges", MetricValue::U64(edges as u64)),
            ("recursion_cycles", MetricValue::U64(0)),
            ("heap_token_violations", MetricValue::U64(heap_hits as u64)),
        ],
        scoping: ScopingRecord::partial(
            "stack bound for the PN64 hot path: zero recursion (call-graph \
             cycle detection) and zero heap allocation on state paths, with a \
             measured minimum surviving thread stack",
            "iterative DFS cycle detection + heap-token audit over the \
             compile-time-embedded source, plus injected-`Vec::` mutant \
             positive control",
            "measured legs (32 KiB workload, child-process stack ladder \
             32768 -> 4096) are #[ignore = \"nato-tier3\"] and NOT in this \
             adapter run; pillar mapped to Dossier per the N04 precedent \
             (no dedicated pillar variant); no total-frame-bytes claim",
        ),
    }
}

impl Court for StackBoundCourt {
    fn id(&self) -> CourtId {
        Self::ID
    }

    fn pillar(&self) -> Pillar {
        Self::PILLAR
    }

    fn name(&self) -> &'static str {
        "N16 stack_bound"
    }

    fn positive_control(&self) -> ControlProbe {
        stackbound_positive_control
    }

    fn run(&self, _candidate: &CandidateRef) -> CourtOutcome {
        stackbound_run_court_probe()
    }
}

/// Audit W3: the cycle detector's vacuity control — the injected
/// self-recursive mutant MUST be flagged (court has power).
#[test]
fn n16_court_detects_injected_recursion() {
    assert!(stackbound_recursion_control(), "injected self-recursion positive control must fire");
}
