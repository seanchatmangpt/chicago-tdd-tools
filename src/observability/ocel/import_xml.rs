//! OCEL 2.0 XML import — roundtrip-first reader for the exported envelope.
//!
//! **Roundtrip-first reader: accepts exactly the shape the ocel-xml export
//! emits; anything else is refused ([`XmlImportError::MalformedXml`]), NOT a
//! general OCEL 2.0 XML parser — disclosed scope.**
//!
//! Accepted envelope (mirroring the exporter in
//! `strata_protocol/src/ocel_xml.rs` — events with `id`/`timestamp`/`activity`
//! attributes, `<object>` children, five predefined entities only):
//!
//! ```xml
//! <ocel:events version="2.0">
//!   <event id="e_0" timestamp="9" activity="TestStarted">
//!     <object id="o_1" type="Artifact"/>
//!   </event>
//! </ocel:events>
//! ```
//!
//! * the escaper inverse covers exactly `&amp; &lt; &gt; &quot; &apos;` — a
//!   numeric or unknown entity is a hard refusal, never a silent passthrough.
//! * `activity` / `type` attribute values are deserialized as
//!   [`TestActivity`]/[`TestObjectType`] variant names via serde; an unknown
//!   variant is refused, not coerced. Unit variants only — payload-carrying
//!   variants (e.g. `TestCompleted { passed }`) are not expressible as a bare
//!   attribute string and are refused with the rest of the malformed shape.
//! * hand-rolled scanner, no XML crate — same offline tier as
//!   [`super::import_json`].
//!
//! [`TestActivity`]: super::types::TestActivity
//! [`TestObjectType`]: super::types::TestObjectType

#![cfg(feature = "ocel-ingest-xml")]

use super::types::{OcelLog, TestObjectType, TestOcelEvent};
use std::collections::HashMap;
use thiserror::Error;

/// Typed refusal surface for OCEL XML import. Fail-closed: any deviation from
/// the exported shape refuses the whole document, never a partial import.
#[derive(Debug, Clone, Error)]
pub enum XmlImportError {
    /// Input deviated from the exported shape at the given byte offset.
    #[error("malformed OCEL XML at byte {offset}: {reason}")]
    MalformedXml {
        /// Byte offset of the offending input.
        offset: usize,
        /// What was expected vs. what was found.
        reason: String,
    },
    /// A tag opened but its closer never appeared (or nested wrongly).
    #[error("unclosed tag <{tag}> at byte {offset}")]
    UnclosedTag {
        /// The tag name left open.
        tag: String,
        /// Byte offset where the open tag began.
        offset: usize,
    },
    /// A required attribute was absent from a tag that carries it.
    #[error("missing attribute \"{attr}\" on <{tag}>")]
    MissingAttribute {
        /// The tag whose attribute set was incomplete.
        tag: &'static str,
        /// The attribute name that was absent.
        attr: &'static str,
    },
    /// The root `version` attribute was present but not `"2.0"`.
    #[error("unsupported ocel XML version {found:?} (only \"2.0\" accepted)")]
    InvalidVersion {
        /// The version string that was found.
        found: String,
    },
}

/// Unescape exactly the five predefined XML entities (the inverse of the
/// exporter's escaper). Unknown or numeric entities are hard refusals.
fn unescape(s: &str, offset: usize) -> Result<String, XmlImportError> {
    if !s.contains('&') {
        return Ok(s.to_owned());
    }
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < s.len() {
        if bytes[i] == b'&' {
            let semi = s[i..].find(';').ok_or_else(|| XmlImportError::MalformedXml {
                offset,
                reason: "entity reference without closing ';'".to_string(),
            })?;
            match &s[i + 1..i + semi] {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                other => {
                    return Err(XmlImportError::MalformedXml {
                        offset,
                        reason: format!(
                            "unknown entity &{other}; (only the five predefined \
                                         entities are accepted)"
                        ),
                    })
                }
            }
            i += semi + 1;
        } else {
            let ch_len = s[i..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&s[i..i + ch_len]);
            i += ch_len;
        }
    }
    Ok(out)
}

/// One parsed XML tag: name, unescaped attributes, self-closing flag, and the
/// byte offset where the tag began (for error reporting).
struct Tag {
    name: String,
    attrs: HashMap<String, String>,
    self_closing: bool,
    offset: usize,
}

/// Scanner over the XML source. Hand-rolled, no XML crate.
struct Scanner<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Scanner<'a> {
    const fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn skip_ws(&mut self) {
        let rest = &self.src[self.pos..];
        let trimmed = rest.trim_start();
        self.pos = self.src.len() - trimmed.len();
    }

    // Reads no scanner state; kept as a method for call-site brevity.
    #[allow(clippy::unused_self)]
    fn err(&self, offset: usize, reason: impl Into<String>) -> XmlImportError {
        XmlImportError::MalformedXml { offset, reason: reason.into() }
    }

    /// Expect and consume a literal, or refuse.
    fn expect_lit(&mut self, lit: &str, what: &str) -> Result<(), XmlImportError> {
        if self.src[self.pos..].starts_with(lit) {
            self.pos += lit.len();
            Ok(())
        } else {
            Err(self.err(self.pos, format!("expected {what} ({lit:?})")))
        }
    }

    /// Parse one tag beginning at the current `<`. Returns [`None`] at end of
    /// input (caller converts to [`XmlImportError::UnclosedTag`] where a
    /// closer was required).
    fn next_tag(&mut self) -> Result<Option<Tag>, XmlImportError> {
        self.skip_ws();
        if self.pos >= self.src.len() {
            return Ok(None);
        }
        let offset = self.pos;
        self.expect_lit("<", "tag start '<'")?;
        // A leading '/' is part of the tag name (closing tag) — the caller
        // dispatches on names like "/ocel:events" / "/event".
        let had_slash = self.src.as_bytes().get(self.pos) == Some(&b'/');
        if had_slash {
            self.pos += 1;
        }
        let name_start = self.pos;
        while self.pos < self.src.len()
            && !matches!(self.src.as_bytes()[self.pos], b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r')
        {
            self.pos += 1;
        }
        let mut name = self.src[name_start..self.pos].to_owned();
        if had_slash {
            name.insert(0, '/');
        }
        if name.is_empty() {
            return Err(self.err(offset, "empty tag name"));
        }
        let mut attrs = HashMap::new();
        loop {
            self.skip_ws();
            if self.pos >= self.src.len() {
                return Err(XmlImportError::UnclosedTag { tag: name, offset });
            }
            match self.src.as_bytes()[self.pos] {
                b'/' => {
                    self.pos += 1;
                    self.expect_lit(">", "tag end '>' after '/'")?;
                    return Ok(Some(Tag { name, attrs, self_closing: true, offset }));
                }
                b'>' => {
                    self.pos += 1;
                    return Ok(Some(Tag { name, attrs, self_closing: false, offset }));
                }
                _ => {
                    let a_start = self.pos;
                    while self.pos < self.src.len()
                        && !matches!(
                            self.src.as_bytes()[self.pos],
                            b'=' | b' ' | b'>' | b'/' | b'\t' | b'\n' | b'\r'
                        )
                    {
                        self.pos += 1;
                    }
                    let attr_name = self.src[a_start..self.pos].to_owned();
                    if attr_name.is_empty() {
                        return Err(self.err(offset, "empty attribute name"));
                    }
                    self.skip_ws();
                    self.expect_lit("=", "'=' after attribute name")?;
                    self.skip_ws();
                    self.expect_lit("\"", "opening '\"' of attribute value")?;
                    let v_start = self.pos;
                    while self.pos < self.src.len() && self.src.as_bytes()[self.pos] != b'"' {
                        self.pos += 1;
                    }
                    if self.pos >= self.src.len() {
                        return Err(XmlImportError::UnclosedTag { tag: name, offset });
                    }
                    let raw = &self.src[v_start..self.pos];
                    self.pos += 1; // closing quote
                    let value = unescape(raw, offset)?;
                    if attrs.insert(attr_name.clone(), value).is_some() {
                        return Err(
                            self.err(a_start, format!("duplicate attribute \"{attr_name}\""))
                        );
                    }
                }
            }
        }
    }
}

/// Import an OCEL 2.0 XML document (exported envelope shape only) into the
/// in-crate [`OcelLog`].
///
/// Fails closed: refuses the whole document on any deviation from the
/// exported shape — never partially imports.
///
/// # Errors
/// Returns [`XmlImportError`] on malformed shape, unclosed tags, missing
/// required attributes, an unsupported `version`, or an unknown
/// activity/object-type variant. See the module-level disclosed scope.
pub fn import_ocel_xml(xml: &str) -> Result<OcelLog, XmlImportError> {
    let mut sc = Scanner::new(xml);

    // ── Root: <ocel:events version="2.0"> ────────────────────────────────────
    let root = sc.next_tag()?.ok_or_else(|| XmlImportError::MalformedXml {
        offset: 0,
        reason: "empty document: expected <ocel:events>".to_string(),
    })?;
    if root.name != "ocel:events" {
        return Err(
            sc.err(root.offset, format!("expected root <ocel:events>, found <{}>", root.name))
        );
    }
    // Version check precedes the self-closing check so a wrong version is
    // reported as InvalidVersion even on a self-closing root.
    let version = root
        .attrs
        .get("version")
        .ok_or(XmlImportError::MissingAttribute { tag: "ocel:events", attr: "version" })?;
    if version != "2.0" {
        return Err(XmlImportError::InvalidVersion { found: version.clone() });
    }
    if root.self_closing {
        return Err(sc.err(root.offset, "root <ocel:events/> cannot carry events"));
    }

    let mut log = OcelLog::new();

    // ── Events ──────────────────────────────────────────────────────────────
    loop {
        let Some(tag) = sc.next_tag()? else {
            return Err(XmlImportError::UnclosedTag {
                tag: "ocel:events".to_owned(),
                offset: root.offset,
            });
        };
        match tag.name.as_str() {
            "/ocel:events" => break, // closer consumed by next_tag name scan
            "/event" | "/object" => {
                return Err(sc.err(tag.offset, format!("unexpected closer </{}>", tag.name)))
            }
            "event" => {
                let ev = parse_event(&mut sc, &tag)?;
                if log.events.insert(ev.event_id.clone(), ev).is_some() {
                    return Err(sc.err(
                        tag.offset,
                        format!(
                            "duplicate event id \"{}\"",
                            tag.attrs.get("id").map_or("", String::as_str)
                        ),
                    ));
                }
            }
            other => {
                return Err(sc.err(
                    tag.offset,
                    format!("unexpected element <{other}> (only <event> allowed)"),
                ))
            }
        }
    }

    sc.skip_ws();
    if sc.pos != sc.src.len() {
        return Err(sc.err(sc.pos, "trailing content after </ocel:events>"));
    }
    Ok(log)
}

/// Parse one `<event>` (with its `<object>` children) into a [`TestOcelEvent`].
fn parse_event(sc: &mut Scanner<'_>, tag: &Tag) -> Result<TestOcelEvent, XmlImportError> {
    let id = tag
        .attrs
        .get("id")
        .ok_or(XmlImportError::MissingAttribute { tag: "event", attr: "id" })?;
    let timestamp_ns: u64 = tag
        .attrs
        .get("timestamp")
        .ok_or(XmlImportError::MissingAttribute { tag: "event", attr: "timestamp" })?
        .parse()
        .map_err(|_| XmlImportError::MalformedXml {
            offset: tag.offset,
            reason: format!("event \"{id}\": timestamp is not a u64"),
        })?;
    let activity_raw = tag
        .attrs
        .get("activity")
        .ok_or(XmlImportError::MissingAttribute { tag: "event", attr: "activity" })?;
    let activity = parse_variant::<super::types::TestActivity>(activity_raw).map_err(|e| {
        XmlImportError::MalformedXml {
            offset: tag.offset,
            reason: format!("event \"{id}\": unknown activity {e}"),
        }
    })?;
    let case_id = tag.attrs.get("caseId").map_or_else(|| id.clone(), std::clone::Clone::clone);

    let mut objects: Vec<(String, TestObjectType)> = Vec::new();
    if !tag.self_closing {
        loop {
            let Some(child) = sc.next_tag()? else {
                return Err(XmlImportError::UnclosedTag {
                    tag: "event".to_owned(),
                    offset: tag.offset,
                });
            };
            match child.name.as_str() {
                "/event" => break,
                "object" => {
                    let oid = child
                        .attrs
                        .get("id")
                        .ok_or(XmlImportError::MissingAttribute { tag: "object", attr: "id" })?;
                    let ty_raw = child
                        .attrs
                        .get("type")
                        .ok_or(XmlImportError::MissingAttribute { tag: "object", attr: "type" })?;
                    let ty = parse_variant::<TestObjectType>(ty_raw).map_err(|e| {
                        XmlImportError::MalformedXml {
                            offset: child.offset,
                            reason: format!("object \"{oid}\": unknown type {e}"),
                        }
                    })?;
                    objects.push((oid.clone(), ty));
                }
                other => {
                    return Err(sc.err(
                        child.offset,
                        format!("unexpected element <{other}> inside <event> (only <object>)"),
                    ))
                }
            }
        }
    }

    Ok(TestOcelEvent {
        event_id: id.clone(),
        case_id,
        activity,
        timestamp_ns,
        objects,
        attributes: HashMap::new(),
    })
}

/// Deserialize a bare variant name into an enum via serde (roundtrip with the
/// exporter's plain-name emission). Returns the serde message on failure.
fn parse_variant<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, String> {
    // Quote it and let serde_json deserialize the variant name; this matches
    // how the export side writes enum names.
    let quoted = format!("\"{raw}\"");
    serde_json::from_str(&quoted).map_err(|e| e.to_string())
}

#[allow(clippy::unwrap_used)] // JUSTIFICATION: Test code only
#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::ocel::types::TestActivity;

    /// Export-shaped fixture: the exact envelope a `to_ocel_xml`-style
    /// exporter emits for a two-event log.
    fn fixture() -> String {
        "<ocel:events version=\"2.0\">\n\
         <event id=\"e_0\" timestamp=\"9\" activity=\"TestStarted\">\n  \
         <object id=\"o_1\" type=\"Artifact\"/>\n  \
         <object id=\"o_2\" type=\"Receipt\"/>\n\
         </event>\n\
         <event id=\"e_1\" timestamp=\"10\" activity=\"DriftZero\" caseId=\"run_7\"/>\n\
         </ocel:events>"
            .to_string()
    }

    #[test]
    fn roundtrip_two_event_fixture() {
        let log = import_ocel_xml(&fixture()).expect("fixture must import");
        assert_eq!(log.events.len(), 2);

        let e0 = log.events.get("e_0").expect("e_0");
        assert_eq!(e0.event_id, "e_0");
        assert_eq!(e0.timestamp_ns, 9);
        assert_eq!(e0.case_id, "e_0");
        assert!(matches!(e0.activity, TestActivity::TestStarted));
        assert_eq!(e0.objects.len(), 2);
        assert_eq!(e0.objects[0], ("o_1".to_string(), TestObjectType::Artifact));
        assert_eq!(e0.objects[1], ("o_2".to_string(), TestObjectType::Receipt));

        let e1 = log.events.get("e_1").expect("e_1");
        assert_eq!(e1.case_id, "run_7");
        assert!(matches!(e1.activity, TestActivity::DriftZero));
        assert!(e1.objects.is_empty());
    }

    #[test]
    fn roundtrip_is_stable_reimport_of_canonical_reshaping() {
        // Import the same fixture twice: byte-identical input must yield
        // byte-identical output state (deterministic reader).
        let a = import_ocel_xml(&fixture()).expect("first import");
        let b = import_ocel_xml(&fixture()).expect("second import");
        assert_eq!(a.events.len(), b.events.len());
        for (id, ev) in &a.events {
            let other = b.events.get(id).expect("same id set");
            assert_eq!(ev.event_id, other.event_id);
            assert_eq!(ev.timestamp_ns, other.timestamp_ns);
            assert_eq!(ev.objects, other.objects);
        }
    }

    #[test]
    fn entity_unescape_roundtrip_all_five() {
        let xml = concat!(
            "<ocel:events version=\"2.0\">",
            "<event id=\"e_&amp;y&lt;z&gt;w&quot;v&apos;a&apos;s\" ",
            "timestamp=\"0\" activity=\"DriftZero\"/>",
            "</ocel:events>"
        );
        let log = import_ocel_xml(xml).expect("entities must unescape");
        let ev = log.events.iter().next().expect("one event").1;
        assert_eq!(ev.event_id, "e_&y<z>w\"v'a's");
    }

    #[test]
    fn unknown_entity_is_refused_not_passthrough() {
        let xml = "<ocel:events version=\"2.0\"><event id=\"&#65;\" timestamp=\"0\" \
                   activity=\"DriftZero\"/></ocel:events>";
        let err = import_ocel_xml(xml).expect_err("numeric entity refused");
        assert!(matches!(err, XmlImportError::MalformedXml { .. }));
        assert!(err.to_string().contains("unknown entity"));
    }

    #[test]
    fn missing_version_attr_refused() {
        let err = import_ocel_xml(
            "<ocel:events><event id=\"e\" timestamp=\"0\" \
                                   activity=\"DriftZero\"/></ocel:events>",
        )
        .expect_err("missing version");
        assert!(matches!(
            err,
            XmlImportError::MissingAttribute { tag: "ocel:events", attr: "version" }
        ));
    }

    #[test]
    fn wrong_version_refused() {
        let err = import_ocel_xml("<ocel:events version=\"1.0\"/>").expect_err("bad version");
        assert!(matches!(err, XmlImportError::InvalidVersion { found } if found == "1.0"));
    }

    #[test]
    fn missing_event_attributes_refused() {
        for (xml, attr) in [
            (
                "<ocel:events version=\"2.0\"><event timestamp=\"0\" activity=\"DriftZero\"/>\
              </ocel:events>",
                "id",
            ),
            (
                "<ocel:events version=\"2.0\"><event id=\"e\" activity=\"DriftZero\"/>\
              </ocel:events>",
                "timestamp",
            ),
            (
                "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"0\"/>\
              </ocel:events>",
                "activity",
            ),
        ] {
            let err = import_ocel_xml(xml).expect_err(attr);
            assert!(
                matches!(&err, XmlImportError::MissingAttribute { tag: "event", attr: a } if *a == attr),
                "{err}"
            );
        }
    }

    #[test]
    fn missing_object_attributes_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"0\" \
             activity=\"DriftZero\"><object id=\"o_1\"/></event></ocel:events>",
        )
        .expect_err("object missing type");
        assert!(matches!(err, XmlImportError::MissingAttribute { tag: "object", attr: "type" }));
    }

    #[test]
    fn unclosed_root_refused() {
        let err = import_ocel_xml("<ocel:events version=\"2.0\">").expect_err("unclosed root");
        assert!(matches!(err, XmlImportError::UnclosedTag { tag, .. } if tag == "ocel:events"));
    }

    #[test]
    fn unclosed_event_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"0\" \
             activity=\"DriftZero\">",
        )
        .expect_err("unclosed event");
        assert!(matches!(err, XmlImportError::UnclosedTag { tag, .. } if tag == "event"));
    }

    #[test]
    fn unclosed_attribute_quote_refused() {
        let err = import_ocel_xml("<ocel:events version=\"2.0\"><event id=\"e ")
            .expect_err("unclosed quote");
        assert!(matches!(err, XmlImportError::UnclosedTag { tag, .. } if tag == "event"));
    }

    #[test]
    fn unexpected_element_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><objects/><event id=\"e\" timestamp=\"0\" \
             activity=\"DriftZero\"/></ocel:events>",
        )
        .expect_err("stray element");
        assert!(err.to_string().contains("unexpected element <objects>"));
    }

    #[test]
    fn unknown_activity_variant_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"0\" \
             activity=\"NotARealActivity\"/></ocel:events>",
        )
        .expect_err("unknown variant");
        assert!(err.to_string().contains("unknown activity"));
    }

    #[test]
    fn unknown_object_type_variant_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"0\" \
             activity=\"DriftZero\"><object id=\"o\" type=\"Banana\"/></event>\
             </ocel:events>",
        )
        .expect_err("unknown object type");
        assert!(err.to_string().contains("unknown type"));
    }

    #[test]
    fn non_u64_timestamp_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"-3\" \
             activity=\"DriftZero\"/></ocel:events>",
        )
        .expect_err("negative timestamp");
        assert!(err.to_string().contains("not a u64"));
    }

    #[test]
    fn duplicate_event_id_refused() {
        let ev = "<event id=\"e_0\" timestamp=\"0\" activity=\"DriftZero\"/>";
        let err = import_ocel_xml(&format!("<ocel:events version=\"2.0\">{ev}{ev}</ocel:events>"))
            .expect_err("duplicate id");
        assert!(err.to_string().contains("duplicate event id"));
    }

    #[test]
    fn trailing_content_refused() {
        let err = import_ocel_xml(
            "<ocel:events version=\"2.0\"><event id=\"e\" timestamp=\"0\" \
             activity=\"DriftZero\"/></ocel:events> garbage",
        )
        .expect_err("trailing content");
        assert!(err.to_string().contains("trailing content"));
    }

    #[test]
    fn wrong_root_element_refused() {
        let err = import_ocel_xml("<log version=\"2.0\"></log>").expect_err("wrong root");
        assert!(err.to_string().contains("expected root <ocel:events>"));
    }

    #[test]
    fn empty_document_refused() {
        let err = import_ocel_xml("").expect_err("empty");
        assert!(err.to_string().contains("empty document"));
    }
}
