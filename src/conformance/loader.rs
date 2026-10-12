//! Fail-closed `LangSec` loaders for the conformance subsystem.
//!
//! Parse, don't validate: every byte of input is either accepted into a typed
//! [`PowlModel`]/[`PetriNet`] or rejected with a positioned
//! [`ConformanceError`]. No regex dependency; a hand-rolled byte scanner
//! tracks byte offsets, and a small `line_col` helper converts them to
//! 1-based line/column so every rejection names its location.
//!
//! # POWL compact DSL
//!
//! ```text
//! activity handle = log.handle
//! activity fork   = robot.fork
//!
//! sequence {
//!     handle
//!     xor {
//!         fork
//!         silent
//!     }
//! }
//! ```
//!
//! Declarations bind `name = source.event`. The root is a single block
//! (`sequence` / `xor` / `parallel` / `loop`), or a leaf (`silent`, or a
//! declared activity name). A `loop` block is written `loop { body }
//! redo { handler } max N`; the `redo` and `max` clauses may be omitted
//! (`max` defaults to 255). Blocks nest arbitrarily. Comments run from `#`
//! to end of line. Unknown keywords, unbalanced braces, undeclared names,
//! duplicate declarations, non-numeric coordinates, and `max` out of
//! `0..=255` are each a typed [`ConformanceError`].
//!
//! Nothing in this module panics; all failure is typed.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::error::ConformanceError;

/// Maximum number of nodes a [`PowlModel`] may contain.
///
/// Keeps recursive traversal bounded; [`PowlModel::from_powl_str`] enforces
/// this so an oversized source is rejected before any traversal.
pub const MAX_POWL_NODES: usize = 64;

// ---------------------------------------------------------------------------
// Model types
// ---------------------------------------------------------------------------

/// A POWL process model: declared activities plus one root expression.
///
/// Serde derives make the in-memory model the single source of truth for both
/// the compact DSL and the JSON surface — the DSL is syntax for this value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowlModel {
    /// Declared activities keyed by name.
    pub activities: BTreeMap<String, ActivityDecl>,
    /// Names in declaration order.
    pub declaration_order: Vec<String>,
    /// The root POWL expression.
    pub root: PowlNode,
}

/// A declared activity: a name bound to a source coordinate `source.event`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityDecl {
    /// Source system of the activity's events.
    pub source: String,
    /// Event class within the source.
    pub event: String,
}

/// A POWL expression node.
///
/// Serde support is hand-written (not derived): an internally-tagged enum
/// cannot carry a newtype variant wrapping a sequence or a plain string
/// (`cannot serialize tagged newtype variant ... containing a sequence`).
/// The hand-written impls give every variant a `kind` tag with `snake_case`
/// spelling (`{"kind":"sequence","children":[...]}`,
/// `{"kind":"activity","name":"x"}`) and round-trip exactly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowlNode {
    /// Total order over child expressions.
    Sequence(Vec<Self>),
    /// Exclusive choice among child expressions.
    Xor(Vec<Self>),
    /// Concurrency among child expressions.
    Parallel(Vec<Self>),
    /// Loop: run `body`, then exit or run `redo`; `max` bounds body
    /// executions in `0..=255`.
    Loop {
        /// Body executed each iteration.
        body: Box<Self>,
        /// Redo branch taken instead of exiting.
        redo: Box<Self>,
        /// Maximum body executions (0..=255).
        max: u8,
    },
    /// A declared activity referenced by name.
    Activity(String),
    /// The silent task (tau).
    Silent,
}

// ---------------------------------------------------------------------------
// Hand-written serde for PowlNode (kind-tagged, snake_case)
// ---------------------------------------------------------------------------

impl Serialize for PowlNode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde_json::json;
        let value: serde_json::Value = match self {
            Self::Sequence(children) => json!({ "kind": "sequence", "children": children }),
            Self::Xor(children) => json!({ "kind": "xor", "children": children }),
            Self::Parallel(children) => json!({ "kind": "parallel", "children": children }),
            Self::Loop { body, redo, max } => {
                json!({ "kind": "loop", "body": body, "redo": redo, "max": max })
            }
            Self::Activity(name) => json!({ "kind": "activity", "name": name }),
            Self::Silent => json!({ "kind": "silent" }),
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PowlNode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let kind = value
            .get("kind")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("missing 'kind' tag in POWL node"))?;
        let parsed: Result<Self, String> = match kind {
            "sequence" => {
                serde_json::from_value(take(&value, "children").map_err(serde::de::Error::custom)?)
                    .map(PowlNode::Sequence)
                    .map_err(|e| e.to_string())
            }
            "xor" => {
                serde_json::from_value(take(&value, "children").map_err(serde::de::Error::custom)?)
                    .map(PowlNode::Xor)
                    .map_err(|e| e.to_string())
            }
            "parallel" => {
                serde_json::from_value(take(&value, "children").map_err(serde::de::Error::custom)?)
                    .map(PowlNode::Parallel)
                    .map_err(|e| e.to_string())
            }
            "loop" => {
                let body: Self =
                    serde_json::from_value(take(&value, "body").map_err(serde::de::Error::custom)?)
                        .map_err(serde::de::Error::custom)?;
                let redo: Self =
                    serde_json::from_value(take(&value, "redo").map_err(serde::de::Error::custom)?)
                        .map_err(serde::de::Error::custom)?;
                let max: u8 =
                    serde_json::from_value(take(&value, "max").map_err(serde::de::Error::custom)?)
                        .map_err(serde::de::Error::custom)?;
                Ok(Self::Loop { body: Box::new(body), redo: Box::new(redo), max })
            }
            "activity" => {
                serde_json::from_value(take(&value, "name").map_err(serde::de::Error::custom)?)
                    .map(PowlNode::Activity)
                    .map_err(|e| e.to_string())
            }
            "silent" => Ok(Self::Silent),
            other => Err(format!("unknown POWL node kind '{other}'")),
        };
        parsed.map_err(serde::de::Error::custom)
    }
}

/// Takes the given field out of a JSON object, failing with a message when
/// the field is missing (helper for the hand-written [`PowlNode`]
/// deserialization above).
fn take(value: &serde_json::Value, name: &str) -> Result<serde_json::Value, String> {
    value.get(name).cloned().ok_or_else(|| format!("missing '{name}' on POWL node"))
}

// ---------------------------------------------------------------------------
// Byte-offset -> line/col
// ---------------------------------------------------------------------------

/// Convert a byte offset in `src` to a 1-based `(line, col)` pair.
///
/// Lines split on `\n`; columns count bytes (`\r` in CRLF is not counted, so
/// LF and CRLF sources report identical columns).
fn line_col(src: &str, offset: usize) -> (u32, u32) {
    let offset = offset.min(src.len());
    let mut line = 1u32;
    let mut col = 1u32;
    for &b in &src.as_bytes()[..offset] {
        if b == b'\n' {
            line += 1;
            col = 1;
        } else if b != b'\r' {
            col += 1;
        }
    }
    (line, col)
}

/// Build a positioned [`ConformanceError::MalformedSource`] at `offset`.
fn malformed(src: &str, offset: usize, reason: &'static str) -> ConformanceError {
    let (line, col) = line_col(src, offset);
    ConformanceError::MalformedSource { line, col, reason }
}

/// Build a positioned [`ConformanceError::UnknownActivity`] at `offset`.
fn unknown_activity(src: &str, name: impl Into<String>, offset: usize) -> ConformanceError {
    let (line, col) = line_col(src, offset);
    ConformanceError::UnknownActivity { name: name.into(), line, col }
}

// ---------------------------------------------------------------------------
// Scanner
// ---------------------------------------------------------------------------

/// A token with the byte offset where it began.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tok<'a> {
    /// `{`
    LBrace(usize),
    /// `}`
    RBrace(usize),
    /// Keyword or identifier.
    Word(&'a str, usize),
    /// `=`
    Eq(usize),
    /// `.`
    Dot(usize),
    /// Unsigned integer literal in `0..=255`.
    Num(u8, usize),
    /// End of input.
    Eof(usize),
}

/// Byte offset carried by a token.
const fn tok_offset(t: &Tok<'_>) -> usize {
    match t {
        Tok::LBrace(o)
        | Tok::RBrace(o)
        | Tok::Word(_, o)
        | Tok::Eq(o)
        | Tok::Dot(o)
        | Tok::Num(_, o)
        | Tok::Eof(o) => *o,
    }
}

/// Hand-rolled one-pass scanner (no regex).
struct Scanner<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Scanner<'a> {
    const fn new(src: &'a str) -> Self {
        Self { src, bytes: src.as_bytes(), pos: 0 }
    }

    /// Skip whitespace and `#`-to-end-of-line comments.
    fn skip_trivia(&mut self) {
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                b'#' => {
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }
    }

    /// Tokenize the whole source, failing closed on any illegal byte or on a
    /// number literal above 255.
    fn tokenize(mut self) -> Result<Vec<Tok<'a>>, ConformanceError> {
        let mut toks = Vec::new();
        loop {
            self.skip_trivia();
            if self.pos >= self.bytes.len() {
                toks.push(Tok::Eof(self.pos));
                return Ok(toks);
            }
            let start = self.pos;
            match self.bytes[start] {
                b'{' => {
                    self.pos += 1;
                    toks.push(Tok::LBrace(start));
                }
                b'}' => {
                    self.pos += 1;
                    toks.push(Tok::RBrace(start));
                }
                b'=' => {
                    self.pos += 1;
                    toks.push(Tok::Eq(start));
                }
                b'.' => {
                    self.pos += 1;
                    toks.push(Tok::Dot(start));
                }
                b'0'..=b'9' => {
                    let mut val: u16 = 0;
                    while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                        val = val * 10 + u16::from(self.bytes[self.pos] - b'0');
                        if val > 255 {
                            return Err(malformed(self.src, start, "number literal exceeds 255"));
                        }
                        self.pos += 1;
                    }
                    // JUSTIFICATION: val <= 255 is enforced by the loop
                    // above, so the u16 -> u8 narrowing is total here.
                    #[allow(clippy::cast_possible_truncation)]
                    let val8 = val as u8;
                    toks.push(Tok::Num(val8, start));
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                    while self.pos < self.bytes.len()
                        && (self.bytes[self.pos].is_ascii_alphanumeric()
                            || self.bytes[self.pos] == b'_')
                    {
                        self.pos += 1;
                    }
                    toks.push(Tok::Word(&self.src[start..self.pos], start));
                }
                _ => {
                    return Err(malformed(self.src, start, "illegal character in source"));
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Words that may not be used as declared activity / source / event names.
fn is_reserved(w: &str) -> bool {
    matches!(w, "activity" | "sequence" | "xor" | "parallel" | "loop" | "redo" | "max" | "silent")
}

/// Recursive-descent parser over the token stream.
struct Parser<'a> {
    src: &'a str,
    toks: Vec<Tok<'a>>,
    pos: usize,
    activities: BTreeMap<String, ActivityDecl>,
    order: Vec<String>,
}

impl<'a> Parser<'a> {
    const fn new(src: &'a str, toks: Vec<Tok<'a>>) -> Self {
        Self { src, toks, pos: 0, activities: BTreeMap::new(), order: Vec::new() }
    }

    fn peek(&self) -> &Tok<'a> {
        // tokenize() always appends a `Tok::Eof`, so `toks` is non-empty and
        // a position past the end clamps to the Eof token. Indexing keeps
        // this expect-free; the only panic case (empty `toks`) is
        // unreachable by construction of tokenize().
        let last = self.toks.len().saturating_sub(1);
        &self.toks[self.pos.min(last)]
    }

    fn bump(&mut self) -> Tok<'a> {
        let t = self.peek().clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    /// Parse the document: zero or more declarations, then one root.
    fn parse_document(&mut self) -> Result<PowlModel, ConformanceError> {
        while let Tok::Word("activity", at) = self.peek() {
            let at = *at;
            self.bump();
            self.parse_activity_decl(at)?;
        }
        if matches!(self.peek(), Tok::Eof(_)) {
            return Err(malformed(self.src, self.eof_offset(), "missing root expression"));
        }
        let root = self.parse_expr()?;
        match self.peek() {
            Tok::Eof(_) => {}
            other => {
                let at = tok_offset(other);
                let reason = if matches!(other, Tok::RBrace(_)) {
                    "unbalanced '}' with no matching '{'"
                } else {
                    "unexpected trailing tokens after root expression"
                };
                return Err(malformed(self.src, at, reason));
            }
        }
        Ok(PowlModel {
            activities: std::mem::take(&mut self.activities),
            declaration_order: std::mem::take(&mut self.order),
            root,
        })
    }

    /// `activity <name> = <source>.<event>`
    fn parse_activity_decl(&mut self, _kw_at: usize) -> Result<(), ConformanceError> {
        let name = match self.bump() {
            Tok::Word(w, at) if !is_reserved(w) => (w, at),
            Tok::Word(_, at) => {
                return Err(malformed(self.src, at, "reserved word used as activity name"));
            }
            other => {
                let at = tok_offset(&other);
                return Err(malformed(self.src, at, "expected activity name after 'activity'"));
            }
        };
        match self.bump() {
            Tok::Eq(_) => {}
            other => {
                let at = tok_offset(&other);
                return Err(malformed(self.src, at, "expected '=' in activity declaration"));
            }
        }
        let source = match self.bump() {
            Tok::Word(w, _) if !is_reserved(w) => w,
            Tok::Word(_, at) => {
                return Err(malformed(self.src, at, "reserved word used as source name"));
            }
            other => {
                let at = tok_offset(&other);
                return Err(malformed(
                    self.src,
                    at,
                    "expected source name in activity declaration",
                ));
            }
        };
        match self.bump() {
            Tok::Dot(_) => {}
            other => {
                let at = tok_offset(&other);
                return Err(malformed(self.src, at, "expected '.' in activity declaration"));
            }
        }
        let event = match self.bump() {
            Tok::Word(w, _) if !is_reserved(w) => w,
            Tok::Word(_, at) => {
                return Err(malformed(self.src, at, "reserved word used as event name"));
            }
            other => {
                let at = tok_offset(&other);
                return Err(malformed(self.src, at, "expected event name in activity declaration"));
            }
        };
        let (line, col) = line_col(self.src, name.1);
        if self
            .activities
            .insert(
                name.0.to_owned(),
                ActivityDecl { source: source.to_owned(), event: event.to_owned() },
            )
            .is_some()
        {
            return Err(ConformanceError::DuplicateActivity { name: name.0.to_owned(), line, col });
        }
        self.order.push(name.0.to_owned());
        Ok(())
    }

    fn expect_lbrace(&mut self, kw_at: usize) -> Result<(), ConformanceError> {
        match self.bump() {
            Tok::LBrace(_) => Ok(()),
            other => {
                let _ = kw_at;
                let at = tok_offset(&other);
                Err(malformed(self.src, at, "expected '{' to open block"))
            }
        }
    }

    /// Parse one expression (block, `silent`, or activity name).
    fn parse_expr(&mut self) -> Result<PowlNode, ConformanceError> {
        match self.bump() {
            Tok::Word("sequence", at) => {
                self.expect_lbrace(at)?;
                let children = self.parse_children()?;
                Ok(PowlNode::Sequence(children))
            }
            Tok::Word("xor", at) => {
                self.expect_lbrace(at)?;
                let children = self.parse_children()?;
                Ok(PowlNode::Xor(children))
            }
            Tok::Word("parallel", at) => {
                self.expect_lbrace(at)?;
                let children = self.parse_children()?;
                Ok(PowlNode::Parallel(children))
            }
            Tok::Word("loop", at) => {
                self.expect_lbrace(at)?;
                let body = self.parse_expr()?;
                self.expect_rbrace()?;
                let redo = if matches!(self.peek(), Tok::Word("redo", _)) {
                    self.bump();
                    self.expect_lbrace(at)?;
                    let node = self.parse_expr()?;
                    self.expect_rbrace()?;
                    node
                } else {
                    PowlNode::Silent
                };
                let max = if matches!(self.peek(), Tok::Word("max", _)) {
                    self.bump();
                    match self.bump() {
                        Tok::Num(n, _) => n,
                        other => {
                            let o = tok_offset(&other);
                            return Err(malformed(self.src, o, "expected number after 'max'"));
                        }
                    }
                } else {
                    255
                };
                Ok(PowlNode::Loop { body: Box::new(body), redo: Box::new(redo), max })
            }
            Tok::Word("silent", _) => Ok(PowlNode::Silent),
            Tok::Word(name, at) => {
                if self.activities.contains_key(name) {
                    Ok(PowlNode::Activity(name.to_owned()))
                } else {
                    Err(unknown_activity(self.src, name, at))
                }
            }
            other => {
                let at = tok_offset(&other);
                Err(malformed(self.src, at, "expected an expression"))
            }
        }
    }

    /// Parse children of a block until the matching `}`.
    fn parse_children(&mut self) -> Result<Vec<PowlNode>, ConformanceError> {
        let mut children = Vec::new();
        loop {
            match self.peek() {
                Tok::RBrace(_) => {
                    self.bump();
                    return Ok(children);
                }
                Tok::Eof(at) => {
                    return Err(malformed(self.src, *at, "unbalanced '{' with no matching '}'"));
                }
                _ => children.push(self.parse_expr()?),
            }
        }
    }

    fn expect_rbrace(&mut self) -> Result<(), ConformanceError> {
        match self.bump() {
            Tok::RBrace(_) => Ok(()),
            other => {
                let at = tok_offset(&other);
                Err(malformed(self.src, at, "expected '}' to close block"))
            }
        }
    }

    fn eof_offset(&self) -> usize {
        match self.toks.last() {
            Some(Tok::Eof(o)) => *o,
            _ => self.src.len(),
        }
    }
}

// ---------------------------------------------------------------------------
// Public API — POWL
// ---------------------------------------------------------------------------

/// Recursively counts nodes in a [`PowlNode`] tree (root included).
fn count_nodes(node: &PowlNode) -> usize {
    match node {
        PowlNode::Activity(_) | PowlNode::Silent => 1,
        PowlNode::Sequence(children) | PowlNode::Xor(children) | PowlNode::Parallel(children) => {
            1 + children.iter().map(count_nodes).sum::<usize>()
        }
        PowlNode::Loop { body, redo, .. } => 1 + count_nodes(body) + count_nodes(redo),
    }
}

/// Depth-first, left-to-right collection of [`PowlNode::Xor`] arities.
fn collect_xor_arities(node: &PowlNode, out: &mut Vec<usize>) {
    match node {
        PowlNode::Activity(_) | PowlNode::Silent => {}
        PowlNode::Sequence(children) | PowlNode::Xor(children) | PowlNode::Parallel(children) => {
            if let PowlNode::Xor(_) = node {
                out.push(children.len());
            }
            for child in children {
                collect_xor_arities(child, out);
            }
        }
        PowlNode::Loop { body, redo, .. } => {
            collect_xor_arities(body, out);
            collect_xor_arities(redo, out);
        }
    }
}

impl PowlModel {
    /// The root expression's total node count (root included).
    #[must_use]
    pub fn node_count(&self) -> usize {
        count_nodes(&self.root)
    }

    /// The arity of each [`PowlNode::Xor`] node, in depth-first
    /// left-to-right order.
    #[must_use]
    pub fn xor_arity_profile(&self) -> Vec<usize> {
        let mut arities = Vec::new();
        collect_xor_arities(&self.root, &mut arities);
        arities
    }

    /// Parse the compact POWL DSL, failing closed with positioned errors.
    ///
    /// # Errors
    ///
    /// Returns [`ConformanceError::ModelTooLarge`] when the parsed tree
    /// contains more than [`MAX_POWL_NODES`] nodes.
    pub fn from_powl_str(src: &str) -> Result<Self, ConformanceError> {
        let toks = Scanner::new(src).tokenize()?;
        let model = Parser::new(src, toks).parse_document()?;
        let found = count_nodes(&model.root);
        if found > MAX_POWL_NODES {
            return Err(ConformanceError::ModelTooLarge { found, limit: MAX_POWL_NODES });
        }
        Ok(model)
    }

    /// Read a file and parse it as the compact POWL DSL.
    ///
    /// # Errors
    ///
    /// Propagates [`ConformanceError::Io`] on an unreadable file and every
    /// parse-time refusal of [`PowlModel::from_powl_str`].
    pub fn from_powl_file(path: impl AsRef<Path>) -> Result<Self, ConformanceError> {
        let path = path.as_ref();
        let src = fs::read_to_string(path).map_err(|e| ConformanceError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        Self::from_powl_str(&src)
    }

    /// Parse POWL model JSON (serde shape of [`PowlModel`]).
    ///
    /// # Errors
    ///
    /// [`ConformanceError::InvalidJson`] when `json` is not valid JSON.
    pub fn from_powl_json(json: &str) -> Result<Self, ConformanceError> {
        serde_json::from_str(json)
            .map_err(|e| ConformanceError::InvalidJson { message: e.to_string() })
    }
}

// ---------------------------------------------------------------------------
// PetriNet
// ---------------------------------------------------------------------------

/// The residual net surface produced by [`ParsedNet::from_net_str`]:
/// a minimal Place/Transition net with plain name-pair arcs.
///
/// Deliberately distinct from [`super::model::PetriNet`], the validated
/// lowering target with typed [`PlaceId`]/[`TransitionId`]-keyed arcs and an
/// initial marking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedNet {
    /// Place names in declaration order.
    pub places: Vec<String>,
    /// Transition names in declaration order.
    pub transitions: Vec<String>,
    /// Directed arcs as `(source, target)` name pairs; `source` is a place
    /// and `target` a transition, or the reverse.
    pub arcs: Vec<(String, String)>,
}

impl ParsedNet {
    /// Parse the minimal net text form. Lines are:
    ///
    /// ```text
    /// place p1
    /// transition t1
    /// arc p1 -> t1
    /// ```
    ///
    /// Blank lines and `#` comments are ignored; anything else is a
    /// positioned [`ConformanceError::MalformedSource`].
    ///
    /// # Errors
    ///
    /// [`ConformanceError::MalformedSource`] on any line that is not one of
    /// the three accepted shapes, and [`ConformanceError::DuplicateActivity`]
    /// on a duplicate place/transition name.
    pub fn from_net_str(src: &str) -> Result<Self, ConformanceError> {
        let mut net = Self { places: Vec::new(), transitions: Vec::new(), arcs: Vec::new() };
        for (idx, raw) in src.lines().enumerate() {
            // JUSTIFICATION: line numbers of real net files fit a u32; the
            // cast saturates for absurd inputs and only affects the error
            // diagnostic, never control flow.
            #[allow(clippy::cast_possible_truncation)]
            let line_no = idx as u32 + 1;
            let line = raw.split('#').next().unwrap_or(raw);
            let parts: Vec<&str> = line.split_whitespace().collect();
            match parts.as_slice() {
                [] => {}
                ["place", name] => {
                    if net.places.iter().any(|p| p == name) {
                        return Err(ConformanceError::DuplicateActivity {
                            name: (*name).to_owned(),
                            line: line_no,
                            col: 7,
                        });
                    }
                    net.places.push((*name).to_owned());
                }
                ["transition", name] => {
                    if net.transitions.iter().any(|t| t == name) {
                        return Err(ConformanceError::DuplicateActivity {
                            name: (*name).to_owned(),
                            line: line_no,
                            col: 12,
                        });
                    }
                    net.transitions.push((*name).to_owned());
                }
                ["arc", from, "->", to] => {
                    net.arcs.push(((*from).to_owned(), (*to).to_owned()));
                }
                _ => {
                    return Err(ConformanceError::MalformedSource {
                        line: line_no,
                        col: 1,
                        reason: "expected 'place <name>', 'transition <name>', or 'arc a -> b'",
                    });
                }
            }
        }
        Ok(net)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a [`PowlModel`] directly (fields are public) around `root`.
    fn model_with_root(root: PowlNode) -> PowlModel {
        PowlModel { activities: BTreeMap::new(), declaration_order: Vec::new(), root }
    }

    #[test]
    fn node_counts_root_and_children() {
        let root = PowlNode::Sequence(vec![
            PowlNode::Activity("a0".to_string()),
            PowlNode::Silent,
            PowlNode::Xor(vec![
                PowlNode::Activity("a1".to_string()),
                PowlNode::Activity("a2".to_string()),
            ]),
        ]);
        let model = model_with_root(root);
        assert_eq!(model.node_count(), 6);
    }

    #[test]
    fn xor_arity_profile_visits_nested_xors_in_dfs_order() {
        let root = PowlNode::Xor(vec![
            PowlNode::Activity("a0".to_string()),
            PowlNode::Xor(vec![
                PowlNode::Activity("a1".to_string()),
                PowlNode::Activity("a2".to_string()),
            ]),
        ]);
        let model = model_with_root(root);
        assert_eq!(model.xor_arity_profile(), vec![2, 2]);
    }

    #[test]
    fn loop_children_are_counted_in_dfs_order() {
        let root = PowlNode::Loop {
            body: Box::new(PowlNode::Sequence(vec![
                PowlNode::Activity("a0".to_string()),
                PowlNode::Silent,
                PowlNode::Activity("a1".to_string()),
            ])),
            redo: Box::new(PowlNode::Activity("a2".to_string())),
            max: 3,
        };
        let model = model_with_root(root);
        // Loop body/redo contribute to the node count like any other child:
        // loop(1) + sequence(1) + a0 + silent + a1 + redo a2 = 6.
        assert_eq!(model.node_count(), 6);
        assert_eq!(model.xor_arity_profile(), Vec::<usize>::new());
    }

    #[test]
    fn oversized_sources_are_rejected() {
        // MAX_POWL_NODES activities in one sequence: the sequence root plus
        // 64 activity leaves = 65 nodes, one over the cap.
        let count = MAX_POWL_NODES;
        let mut src = String::new();
        for i in 0..count {
            src.push_str(&format!("activity a{i} = s.e{i}\n"));
        }
        src.push_str("sequence { ");
        for i in 0..count {
            if i > 0 {
                src.push(' ');
            }
            src.push_str(&format!("a{i}"));
        }
        src.push_str(" }");
        let err = PowlModel::from_powl_str(&src).expect_err("must reject oversized tree");
        match err {
            ConformanceError::ModelTooLarge { found, limit } => {
                assert_eq!(found, MAX_POWL_NODES + 1);
                assert_eq!(limit, MAX_POWL_NODES);
            }
            other => panic!("expected ModelTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn at_capacity_sources_are_admitted() {
        // Exactly MAX_POWL_NODES nodes: the cap is inclusive.
        let count = MAX_POWL_NODES - 1; // sequence root + count activities = 64
        let mut src = String::new();
        for i in 0..count {
            src.push_str(&format!("activity a{i} = s.e{i}\n"));
        }
        src.push_str("sequence { ");
        for i in 0..count {
            if i > 0 {
                src.push(' ');
            }
            src.push_str(&format!("a{i}"));
        }
        src.push_str(" }");
        let model = PowlModel::from_powl_str(&src).expect("at-capacity tree must parse");
        assert_eq!(model.node_count(), MAX_POWL_NODES);
    }

    #[test]
    fn powl_node_serializes_with_snake_case_kind_tag() {
        let json = serde_json::to_string(&PowlNode::Activity("x".to_string()))
            .unwrap_or_else(|e| panic!("serialize: {e}"));
        assert!(json.contains(r#""kind":"activity""#), "unexpected JSON: {json}");
        let round_trip: PowlNode =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("deserialize: {e}"));
        assert_eq!(round_trip, PowlNode::Activity("x".to_string()));
    }

    // Leading/trailing newlines are intentional; the DSL parser skips
    // surrounding whitespace, so no `\`-continuation is needed (a `\` at
    // end-of-line trips the `multiple-lines-skipped-by-escaped-newline`
    // warning, which is denied under `#![deny(warnings)]`).
    const GOLDEN: &str = "

activity handle = log.handle
activity fork = robot.fork
activity join = robot.join

sequence {
    handle
    xor {
        fork
        parallel {
            join
            silent
        }
    }
    loop {
        handle
    } redo {
        silent
    } max 7
}
";

    #[test]
    fn golden_dsl_parses() {
        let model = PowlModel::from_powl_str(GOLDEN).expect("golden source must parse");
        assert_eq!(model.declaration_order, vec!["handle", "fork", "join"]);
        assert_eq!(
            model.activities["fork"],
            ActivityDecl { source: "robot".into(), event: "fork".into() }
        );
        assert!(matches!(model.root, PowlNode::Sequence(ref c) if c.len() == 3));
        // Loop node carried its max.
        let seq = match &model.root {
            PowlNode::Sequence(c) => c,
            _ => panic!("root must be a sequence"),
        };
        match &seq[2] {
            PowlNode::Loop { max, .. } => assert_eq!(*max, 7),
            other => panic!("expected loop, got {other:?}"),
        }
    }

    #[test]
    fn unknown_word_in_expression_position_is_unknown_activity() {
        // A bare undeclared word (root or nested) resolves through the
        // activity-reference arm: UnknownActivity, positioned at the word.
        let err = PowlModel::from_powl_str("frobnicate { silent }").unwrap_err();
        match err {
            ConformanceError::UnknownActivity { name, line, col } => {
                assert_eq!(name, "frobnicate");
                assert_eq!((line, col), (1, 1));
            }
            other => panic!("expected UnknownActivity, got {other:?}"),
        }
    }

    #[test]
    fn unbalanced_brace_is_malformed() {
        let err = PowlModel::from_powl_str("sequence { silent").unwrap_err();
        assert!(matches!(err, ConformanceError::MalformedSource { .. }));
        let err = PowlModel::from_powl_str("sequence { silent } }").unwrap_err();
        assert!(matches!(err, ConformanceError::MalformedSource { .. }));
    }

    #[test]
    fn undeclared_name_is_unknown_activity() {
        let err = PowlModel::from_powl_str("sequence { ghost }").unwrap_err();
        match err {
            ConformanceError::UnknownActivity { name, line, col } => {
                assert_eq!(name, "ghost");
                assert_eq!((line, col), (1, 12));
            }
            other => panic!("expected UnknownActivity, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_activity_is_typed() {
        let src = "activity a = s.e\nactivity a = s.f\nsequence { a }";
        let err = PowlModel::from_powl_str(src).unwrap_err();
        match err {
            ConformanceError::DuplicateActivity { name, line, col } => {
                assert_eq!(name, "a");
                assert_eq!((line, col), (2, 10));
            }
            other => panic!("expected DuplicateActivity, got {other:?}"),
        }
    }

    #[test]
    fn non_numeric_coord_is_malformed() {
        let err = PowlModel::from_powl_str("activity a = s.e.x\nsequence { a }").unwrap_err();
        assert!(matches!(err, ConformanceError::MalformedSource { .. }));
        let err = PowlModel::from_powl_str("activity a = s.42\nsequence { a }").unwrap_err();
        assert!(matches!(err, ConformanceError::MalformedSource { .. }));
    }

    #[test]
    fn max_out_of_range_is_malformed() {
        let err = PowlModel::from_powl_str("loop { silent } max 256").unwrap_err();
        match err {
            ConformanceError::MalformedSource { line, col, reason } => {
                assert_eq!((line, col), (1, 21));
                assert!(reason.contains("255"));
            }
            other => panic!("expected MalformedSource, got {other:?}"),
        }
    }

    #[test]
    fn json_roundtrip() {
        let model = PowlModel::from_powl_str(GOLDEN).expect("golden parses");
        let json = serde_json::to_string(&model).expect("serialize");
        let back = PowlModel::from_powl_json(&json).expect("deserialize");
        assert_eq!(model, back);
    }

    #[test]
    fn invalid_json_is_typed() {
        let err = PowlModel::from_powl_json("{ not json").unwrap_err();
        assert!(matches!(err, ConformanceError::InvalidJson { .. }));
    }

    #[test]
    fn io_error_names_path() {
        let err = PowlModel::from_powl_file("/definitely/not/a/real/path.powl").unwrap_err();
        match err {
            ConformanceError::Io { path, .. } => {
                assert!(path.contains("definitely/not/a/real/path.powl"));
            }
            other => panic!("expected Io, got {other:?}"),
        }
    }

    #[test]
    fn line_col_helper_tracks_newlines() {
        assert_eq!(line_col("abc", 0), (1, 1));
        assert_eq!(line_col("abc", 2), (1, 3));
        assert_eq!(line_col("ab\ncd", 4), (2, 2));
        assert_eq!(line_col("ab\r\ncd", 5), (2, 2));
        assert_eq!(line_col("ab", 99), (1, 3)); // clamped
    }

    #[test]
    fn net_str_parses_minimal_net() {
        let net = ParsedNet::from_net_str(
            "# a tiny net\nplace p1\nplace p2\ntransition t1\narc p1 -> t1\narc t1 -> p2\n",
        )
        .expect("net parses");
        assert_eq!(net.places, vec!["p1", "p2"]);
        assert_eq!(net.transitions, vec!["t1"]);
        assert_eq!(net.arcs, vec![("p1".into(), "t1".into()), ("t1".into(), "p2".into())]);
    }

    #[test]
    fn net_str_rejects_junk_lines() {
        let err = ParsedNet::from_net_str("frob p1").unwrap_err();
        assert!(matches!(err, ConformanceError::MalformedSource { line: 1, .. }));
    }
}
