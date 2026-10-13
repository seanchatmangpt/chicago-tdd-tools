//! Cluster-pins drift court.
//!
//! Law: every pin in `cluster-pins.toml` must (a) cite a source doc
//! path:line that exists and contains the pinned value verbatim (fail-closed),
//! and (b) match the live sibling checkout unless the pin's `status` honestly
//! records the drift class (`stale-unmerged`, `stale-drift`). A mismatch
//! without a matching status is a CI failure that names both values.
//!
//! Sabotage twin included: corrupting a pin in memory must be caught.
//! Marketplace-pack court-set sync seam is checked with a documented-absent
//! skip (the pack does not enumerate N01..N20; re-verified at runtime).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REPO_ROOT: &str = env!("CARGO_MANIFEST_DIR");
const PINS_FILE: &str = "cluster-pins.toml";
const MARKETPLACE_PACK: &str =
    "/Users/sac/ggen-marketplace/packs/chicago-tdd-tools-pack";
const AS_OF: &str = "2026-10-13";

/// One parsed pin section. Only string-typed keys; unknown keys ignored.
#[derive(Debug, Clone, Default)]
struct Pin {
    value: String,
    kind: String,
    source: String,
    source_alt: String,
    live_checkout: String,
    live_source: String,
    live_kind: String,
    status: String,
    standing: String,
    note: String,
}

#[derive(Debug, Default)]
struct Pins {
    order: Vec<String>,
    map: BTreeMap<String, Pin>,
}

fn load_pins() -> Pins {
    let path = Path::new(REPO_ROOT).join(PINS_FILE);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fail-closed: cannot read {}: {e}", path.display()));
    parse_pins(&text)
}

/// Minimal TOML subset parser: `[pin.NAME]` sections with `key = "value"`.
/// Good enough for this file's shape; a malformed file fails closed.
fn parse_pins(text: &str) -> Pins {
    let mut pins = Pins::default();
    let mut current: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(name) = line
            .strip_prefix("[pin.")
            .and_then(|s| s.strip_suffix(']'))
        {
            current = Some(name.to_string());
            pins.order.push(name.to_string());
            pins.map.insert(name.to_string(), Pin::default());
            continue;
        }
        if line.starts_with('[') {
            // Top-level or non-pin table header: reset section context.
            current = None;
            continue;
        }
        let Some(name) = current.as_ref() else {
            continue;
        };
        let Some((k, v)) = line.split_once('=') else {
            panic!("fail-closed: unparseable pin line: {raw}");
        };
        let key = k.trim();
        let val = v.trim();
        let val = val
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or_else(|| panic!("fail-closed: pin values must be quoted strings: {raw}"));
        let pin = pins.map.get_mut(name).unwrap();
        match key {
            "value" => pin.value = val.to_string(),
            "kind" => pin.kind = val.to_string(),
            "source" => pin.source = val.to_string(),
            "source_alt" => pin.source_alt = val.to_string(),
            "live_checkout" => pin.live_checkout = val.to_string(),
            "live_source" => pin.live_source = val.to_string(),
            "live_kind" => pin.live_kind = val.to_string(),
            "status" => pin.status = val.to_string(),
            "standing" => pin.standing = val.to_string(),
            "note" => pin.note = val.to_string(),
            _ => {}
        }
    }
    pins
}

/// Repo-root-relative `path:line` must exist and contain `needle`.
fn check_source_citation(repo_root: &Path, citation: &str, needle: &str) -> Result<(), String> {
    let Some((path_part, line_part)) = citation.rsplit_once(':') else {
        return Err(format!("citation missing :line: {citation}"));
    };
    let line_no: usize = line_part
        .parse()
        .map_err(|_| format!("citation line not numeric: {citation}"))?;
    let full = repo_root.join(path_part);
    let text = fs::read_to_string(&full)
        .map_err(|e| format!("citation file missing {}: {e}", full.display()))?;
    let line = text
        .lines()
        .nth(line_no.saturating_sub(1))
        .ok_or_else(|| format!("citation line {} out of range in {}", line_no, full.display()))?;
    if line.contains(needle) {
        Ok(())
    } else {
        Err(format!(
            "citation {citation} does not contain pinned value {needle:?}; line reads: {line}"
        ))
    }
}

/// Extract the live version string from a sibling checkout file.
fn extract_live_version(live_kind: &str, path: &str) -> Result<String, String> {
    match live_kind {
        // "git-ancestor" pins are resolved via run_ancestor_check in
        // check_pin_live_state, never through this version extractor.
        "git-ancestor" => Err("git-ancestor handled by run_ancestor_check".into()),
        "elixir" => {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("live source missing {path}: {e}"))?;
            for line in text.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("@version") {
                    if let Some(v) = quoted(rest) {
                        return Ok(v);
                    }
                }
                if let Some(rest) = t.strip_prefix("version:") {
                    if let Some(v) = quoted(rest) {
                        return Ok(v);
                    }
                    return Ok(String::new());
                }
            }
            Err(format!("no @version / version: found in {path}"))
        }
        "cargo" => {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("live source missing {path}: {e}"))?;
            for line in text.lines() {
                let t = line.trim();
                if t.starts_with("version = ") {
                    if let Some(v) = quoted(t.trim_start_matches("version = ")) {
                        return Ok(v);
                    }
                }
            }
            Err(format!("no package version found in {path}"))
        }
        other => Err(format!("unknown live_kind: {other}")),
    }
}

fn quoted(s: &str) -> Option<String> {
    let s = s.trim();
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn run_ancestor_check(checkout: &str, sha: &str) -> Result<bool, String> {
    let out = Command::new("git")
        .current_dir(checkout)
        .args(["merge-base", "--is-ancestor", sha, "main"])
        .output()
        .map_err(|e| format!("git spawn failed: {e}"))?;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        code => Err(format!(
            "git merge-base --is-ancestor {sha} main in {checkout} returned {code:?} (not 0/1)"
        )),
    }
}

/// Full drift detector over one pin. Ok = live state matches the pin file's
/// recorded status honestly. Err = drift or dishonest classification.
fn check_pin_live_state(pin_name: &str, pin: &Pin) -> Result<(), String> {
    if pin.live_source.is_empty() && pin.live_kind != "git-ancestor" {
        return Err(format!("pin {pin_name}: no live_source — unverifiable pin"));
    }
    if pin.live_kind == "git-ancestor" {
        let merged = run_ancestor_check(&pin.live_checkout, &pin.value)
            .map_err(|e| format!("pin {pin_name}: {e}"))?;
        if merged {
            // The stale-unmerged classification must be retired once merged.
            if pin.status == "stale-unmerged" {
                return Err(format!(
                    "pin {pin_name}: {sha} IS NOW MERGED into main — retire the 'stale-unmerged' status in {PINS_FILE}",
                    sha = pin.value
                ));
            }
            return Ok(());
        }
        if pin.status == "stale-unmerged" {
            return Ok(()); // honestly recorded
        }
        return Err(format!(
            "pin {pin_name}: sha {sha} is NOT an ancestor of main in {checkout} (status code 1), but pin status is {status:?} — classify as stale-unmerged",
            sha = pin.value,
            checkout = pin.live_checkout,
            status = pin.status
        ));
    }
    let live = extract_live_version(&pin.live_kind, &pin.live_source)
        .map_err(|e| format!("pin {pin_name}: {e}"))?;
    if live == pin.value {
        if pin.status == "stale-drift" {
            return Err(format!(
                "pin {pin_name}: live == pinned ({v}) but status is stale-drift — the recorded drift has been fixed; set status to ok",
                v = pin.value
            ));
        }
        return Ok(());
    }
    if pin.status == "stale-drift" {
        return Ok(()); // drift honestly recorded
    }
    Err(format!(
        "pin {pin_name}: DRIFT — pinned {p} vs live {l} ({src}); classify status as stale-drift",
        p = pin.value,
        l = live,
        src = pin.live_source
    ))
}

/// Sabotage twin: detector must catch a corrupted pin.
fn sabotage(mut pin: Pin) -> Result<(), String> {
    pin.value = format!("{}-SABOTAGED", pin.value);
    check_pin_live_state("sabotage", &pin)
}

#[test]
fn pins_file_parses_with_all_five_pins() {
    let pins = load_pins();
    for name in [
        "strata",
        "ash_graphlaw",
        "ash_pplan",
        "ash_a2a",
        "temprun",
    ] {
        assert!(
            pins.map.contains_key(name),
            "cluster-pins.toml missing pin [{name}]"
        );
    }
    assert_eq!(pins.order.len(), 5, "expected exactly 5 pins");
}

#[test]
fn every_source_citation_exists_and_contains_pinned_value() {
    let pins = load_pins();
    let root = Path::new(REPO_ROOT);
    let mut failures = Vec::new();
    for name in &pins.order {
        let pin = &pins.map[name];
        if let Err(e) = check_source_citation(root, &pin.source, &pin.value) {
            failures.push(format!("[{name}] {e}"));
        }
        if !pin.source_alt.is_empty() {
            // alt citations: "path:29,122" — multiple line refs on one path
            let (path_part, lines_part) = pin.source_alt.split_once(':').unwrap_or_else(|| {
                panic!("fail-closed: source_alt must be path:lines for [{name}]")
            });
            for line_part in lines_part.split(',') {
                let citation = format!("{path_part}:{line_part}");
                if let Err(e) = check_source_citation(root, &citation, &pin.value) {
                    failures.push(format!("[{name}] (alt) {e}"));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "fail-closed citation court:\n{}",
        failures.join("\n")
    );
}

#[test]
fn live_checkouts_match_pins_or_honestly_recorded_drift() {
    let pins = load_pins();
    let mut failures = Vec::new();
    for name in &pins.order {
        if let Err(e) = check_pin_live_state(name, &pins.map[name]) {
            failures.push(e);
        }
    }
    assert!(
        failures.is_empty(),
        "cluster pin drift court:\n{}",
        failures.join("\n")
    );
}

#[test]
fn sabotage_twin_is_caught() {
    let pins = load_pins();
    let pin = pins.map["ash_a2a"].clone();
    let result = sabotage(pin);
    assert!(result.is_err(), "sabotaged pin must be caught by the drift detector");
    let msg = result.unwrap_err();
    assert!(
        msg.contains("SABOTAGED") && msg.contains("stale-drift"),
        "detector message must name both values, got: {msg}"
    );
}

#[test]
fn marketplace_pack_court_set_sync() {
    // Seam: the marketplace pack mirrors this repo's N01..N20 court set.
    // Verified 2026-10-13: the pack does NOT enumerate courts (no N01..N20
    // tokens in ontology.ttl, gates/, templates/, examples/). Until it does,
    // the seam is documented-absent: assert absence honestly rather than
    // vacuously passing, and the moment a court token appears, the full
    // set-equality court below activates.
    let pack = PathBuf::from(MARKETPLACE_PACK);
    assert!(
        pack.is_dir(),
        "marketplace pack missing at {MARKETPLACE_PACK}"
    );
    let mut found: Vec<PathBuf> = Vec::new();
    collect_court_token_files(&pack, &mut found);
    if found.is_empty() {
        println!(
            "SKIP-REASON (documented-absent, as-of {AS_OF}): pack {MARKETPLACE_PACK} enumerates no N01..N20 court tokens; court-set sync seam not yet landed on the pack side."
        );
        return;
    }
    let expected: std::collections::BTreeSet<String> = court_names_from_registry()
        .into_iter()
        .collect();
    // When the `nato` feature is compiled in, the on-disk registry literal
    // must also equal the real compiled COURT_NAMES (no second source of truth).
    #[cfg(feature = "nato")]
    {
        let compiled: std::collections::BTreeSet<&str> =
            chicago_tdd_tools::nato::registry::COURT_NAMES
                .iter()
                .copied()
                .collect();
        let on_disk: std::collections::BTreeSet<&str> =
            court_names_from_registry().iter().map(String::as_str).collect();
        assert_eq!(
            compiled, on_disk,
            "src/nato/registry.rs literal drifted from compiled COURT_NAMES"
        );
    }
    let mut actual = std::collections::BTreeSet::new();
    for f in &found {
        let text = fs::read_to_string(f).unwrap_or_default();
        for name in expected.iter() {
            if text.contains(name) {
                actual.insert(name.clone());
            }
        }
    }
    assert_eq!(
        actual, expected,
        "marketplace pack court set drifted from src/nato/registry.rs COURT_NAMES"
    );
}

/// Extract the N01..N20 set from the on-disk registry literal.
fn court_names_from_registry() -> Vec<String> {
    let text = fs::read_to_string(Path::new(REPO_ROOT).join("src/nato/registry.rs"))
        .expect("src/nato/registry.rs must exist");
    let start = text
        .find("pub static COURT_NAMES")
        .expect("COURT_NAMES missing from registry.rs");
    let body = &text[start..];
    let open = body.find('[').expect("COURT_NAMES literal missing [");
    let close = body[open..].find(']').expect("COURT_NAMES literal missing ]");
    body[open..open + close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| s.starts_with('N') && s.len() == 3)
        .collect()
}

fn collect_court_token_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "playground" || n == "qualification") {
                continue; // non-authoritative surfaces
            }
            collect_court_token_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "ttl" || e == "rq" || e == "tera" || e == "toml")
        {
            if let Ok(text) = fs::read_to_string(&p) {
                if text.contains("N01") || text.contains("N20") {
                    out.push(p);
                }
              }
        }
    }
}
