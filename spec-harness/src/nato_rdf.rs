//! Receipt → RDF bridge for the 20 NATO courts.
//!
//! Lifts the JSON court receipts emitted by the real runner
//! (`CTT_EMIT_RECEIPTS=1`, `docs/receipts/N{01..20}.json`) into a small
//! Turtle graph so the SPARQL gate set in `spec-harness/run_gates.py` can
//! adjudicate the same verdicts the courts produced.
//!
//! Lift shape (one subject per receipt):
//!
//! ```turtle
//! ct:N01 a ct:CourtReceipt ;
//!     ct:courtId "N01" ;
//!     ct:receiptId "cr-control-0" ;
//!     ct:pillar "Proof" ;
//!     ct:verdict "Pass" ;
//!     ct:sealedDigest "27e3..." ;
//!     ct:timestampMs 0 .
//! ```

use std::fs;
use std::path::Path;

use serde_json::Value;

/// Ontology namespace for the NATO court receipt lift.
pub const NATO_RDF_NS: &str = "http://chicago-tdd-tools.dev/ontology/nato-court#";

/// Fields lifted per receipt. Everything else in the JSON wire format is
/// intentionally not projected — the gates reason over verdict/digest only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiftedReceipt {
    pub court_id: String,
    pub receipt_id: String,
    pub pillar: String,
    pub verdict: String,
    pub sealed_digest: String,
    pub timestamp_ms: u64,
}

/// Parse one receipt JSON object into a [`LiftedReceipt`].
///
/// Typed errors name the missing/invalid field — a malformed receipt is a
/// refusal, never a silent skip.
pub fn parse_receipt(v: &Value) -> Result<LiftedReceipt, String> {
    let obj = v
        .as_object()
        .ok_or_else(|| "receipt is not a JSON object".to_string())?;
    let get_str = |key: &str| -> Result<String, String> {
        obj.get(key)
            .and_then(|x| x.as_str())
            .map(str::to_string)
            .ok_or_else(|| format!("missing or non-string field `{key}`"))
    };
    let timestamp_ms = obj
        .get("timestamp_ms")
        .and_then(|x| x.as_u64())
        .ok_or_else(|| "missing or non-u64 field `timestamp_ms`".to_string())?;
    Ok(LiftedReceipt {
        court_id: get_str("court_id")?,
        receipt_id: get_str("receipt_id")?,
        pillar: get_str("pillar")?,
        verdict: get_str("verdict")?,
        sealed_digest: get_str("digest")?,
        timestamp_ms,
    })
}

/// Lift a directory of court receipts (`N*.json`) to a Turtle string.
///
/// Receipts are emitted sorted by filename (`N01.json` .. `N20.json`), and
/// the lift preserves that order.
pub fn lift_receipts_dir(dir: impl AsRef<Path>) -> Result<String, String> {
    let dir = dir.as_ref();
    let mut paths: Vec<std::path::PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read receipts dir {}: {e}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case("json"))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();

    if paths.is_empty() {
        return Err(format!(
            "no receipt JSON files in {} — run the real court emitter \
             (CTT_EMIT_RECEIPTS=1) first; the lift refuses an empty subject",
            dir.display()
        ));
    }

    let mut turtle = String::from(
        "@prefix ct: <http://chicago-tdd-tools.dev/ontology/nato-court#> .\n\
         @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n",
    );

    for path in &paths {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let v: Value = serde_json::from_str(&json)
            .map_err(|e| format!("invalid JSON in {}: {e}", path.display()))?;
        let r = parse_receipt(&v).map_err(|e| format!("{}: {e}", path.display()))?;
        if r.sealed_digest.len() != 64 || !r.sealed_digest.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(format!(
                "{}: digest is not 64 hex chars — refusing to lift an unsealed receipt",
                path.display()
            ));
        }
        turtle.push_str(&format!(
            "ct:{} a ct:CourtReceipt ;\n    ct:courtId \"{}\" ;\n    ct:receiptId \"{}\" ;\n    ct:pillar \"{}\" ;\n    ct:verdict \"{}\" ;\n    ct:sealedDigest \"{}\" ;\n    ct:timestampMs \"{}\"^^xsd:unsignedLong .\n\n",
            r.court_id, r.court_id, r.receipt_id, r.pillar, r.verdict, r.sealed_digest, r.timestamp_ms
        ));
    }
    Ok(turtle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_receipt_extracts_lifted_fields() {
        let mut json = String::from(
            r#"{"receipt_id":"cr-x","court_id":"N01","pillar":"Proof","name":"N01",
                "subject_sha":"s","verdict":"Pass","metrics":[],
                "positive_control":{"MutantCaught":{"mutant_id":"m","detection_channel":"c"}},
                "scoping":{"spec_mandate":"a","delivered_evidence":"b","is_full_spec_scale":true,"bound":""},
                "digest":""#,
        );
        json.push_str(&"a".repeat(64));
        json.push_str(r#"","timestamp_ms":0}"#);
        let v: Value = serde_json::from_str(&json).expect("json");
        let r = parse_receipt(&v).expect("parses");
        assert_eq!(r.court_id, "N01");
        assert_eq!(r.verdict, "Pass");
        assert_eq!(r.timestamp_ms, 0);
    }

    #[test]
    fn parse_receipt_refuses_missing_fields() {
        let v: Value = serde_json::from_str(r#"{"court_id":"N01"}"#).expect("json");
        let err = parse_receipt(&v).expect_err("must refuse");
        assert!(
            err.contains("missing or non-"),
            "must name the field law, got {err}"
        );
    }

    #[test]
    fn lift_refuses_empty_dir() {
        let tmp = std::env::temp_dir().join(format!("ctt-lift-empty-{}", std::process::id()));
        fs::create_dir_all(&tmp).expect("mkdir");
        let err = lift_receipts_dir(&tmp).expect_err("must refuse");
        assert!(err.contains("no receipt JSON files"));
        fs::remove_dir_all(&tmp).ok();
    }
}
