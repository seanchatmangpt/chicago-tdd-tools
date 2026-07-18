//! syn-based AST scanner for claimed Rust structure.
//!
//! **Evidence class disclosure**: this scanner produces *syntax-level* observations
//! only. It does not prove semantic correctness of any method body, does not resolve
//! macro expansion (a field or method introduced by a macro will not be seen), and
//! does not prove the observed struct/field/method is actually reachable or invoked
//! at runtime. A [`ScanResult`] is raw structural evidence to be reconciled against
//! other evidence classes (mutant-kill oracle tests, runtime traces) — it is never a
//! verdict on its own.

use std::fs;
use std::path::Path;

use std::collections::HashSet;

use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprStruct, Fields, ImplItem, Item, Visibility};
use thiserror::Error;

use crate::claim::RequiredStructure;

/// Structured, syntax-level observations about a claimed Rust structure. This is
/// evidence, not a verdict — see the module-level doc comment for the scope and
/// limits of what it proves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    /// Whether a struct or enum item named `req.type_name` was found in the file.
    pub type_found: bool,
    /// Whether the claimed field's visibility is private (`syn::Visibility::Inherited`).
    /// `None` if the type was not found, or the type has no matching field to check
    /// (e.g. a unit struct, or the named field does not exist).
    pub field_is_private: Option<bool>,
    /// For each name in `req.required_methods`, whether a method of that name was
    /// found in some `impl` block for the claimed type.
    pub methods_found: Vec<(String, bool)>,
    /// For each name in `req.forbidden_constructions`, whether that name's
    /// construction (a call `Name::ctor(..)` or a struct literal `Name { .. }`) was
    /// observed to be ABSENT anywhere in the scanned file. `true` = absent (the
    /// claim's requirement holds); `false` = a construction site was found.
    ///
    /// This is file-wide, not scoped to the claimed type's own `impl` blocks (unlike
    /// `methods_found`): a "never mints X" claim is about the whole module never
    /// constructing X, not about X's own impl block. See the module doc comment for
    /// the evidence-class caveats this still carries (syntax-level only: a
    /// construction reachable only through a macro expansion or a re-exported helper
    /// function defined elsewhere would not be seen here).
    pub forbidden_constructions_absent: Vec<(String, bool)>,
}

/// Syntax-level visitor collecting the set of names, among `targets`, that are
/// observed as the callee of a call expression (`Name::ctor(..)`) or as the type
/// path of a struct-literal expression (`Name { .. }`) anywhere in the visited file.
///
/// Deliberately narrower than "every path mention of `Name`": a return-type
/// annotation, a doc-comment reference, or a parameter type naming `Name` does not
/// count as a construction and must not be flagged, or every function merely
/// *receiving* a `CertificateReceipt` (never minting one) would falsely appear to
/// violate a "never constructs" claim.
struct ForbiddenConstructionVisitor<'a> {
    targets: &'a [String],
    found: HashSet<String>,
}

impl<'a> ForbiddenConstructionVisitor<'a> {
    fn new(targets: &'a [String]) -> Self {
        Self { targets, found: HashSet::new() }
    }

    fn record_if_target(&mut self, ident: &str) {
        if self.targets.iter().any(|t| t == ident) {
            self.found.insert(ident.to_string());
        }
    }
}

impl<'ast, 'a> Visit<'ast> for ForbiddenConstructionVisitor<'a> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(p) = &*node.func {
            if let Some(seg) = p.path.segments.last() {
                self.record_if_target(&seg.ident.to_string());
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        if let Some(seg) = node.path.segments.last() {
            self.record_if_target(&seg.ident.to_string());
        }
        visit::visit_expr_struct(self, node);
    }
}

/// Typed errors for AST scanning. Never panics on malformed or unreadable input.
#[derive(Debug, Error)]
pub enum ScanError {
    /// The scanned file could not be read from disk.
    #[error("failed to read source file {path}: {source}")]
    Io {
        /// Path that failed to read.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The scanned file's contents did not parse as valid Rust source.
    #[error("failed to parse Rust source at {path}: {source}")]
    Parse {
        /// Path that failed to parse.
        path: String,
        /// Underlying syn parse error.
        #[source]
        source: syn::Error,
    },
}

/// Scan `file_path` for a struct/enum matching `req.type_name`, and report structural
/// observations about its claimed field and required methods.
///
/// This is syntax-level evidence only: see the module-level doc comment.
///
/// # Errors
///
/// Returns [`ScanError::Io`] if the file cannot be read, or [`ScanError::Parse`] if
/// the contents do not parse as valid Rust source. Never panics on malformed input.
pub fn scan_required_structure(
    file_path: &Path,
    req: &RequiredStructure,
) -> Result<ScanResult, ScanError> {
    let source = fs::read_to_string(file_path)
        .map_err(|source| ScanError::Io { path: file_path.display().to_string(), source })?;
    let ast = syn::parse_file(&source)
        .map_err(|source| ScanError::Parse { path: file_path.display().to_string(), source })?;

    let mut type_found = false;
    let mut field_is_private: Option<bool> = None;
    let mut methods_seen: Vec<String> = Vec::new();

    for item in &ast.items {
        match item {
            Item::Struct(item_struct) if item_struct.ident == req.type_name => {
                type_found = true;
                field_is_private = observe_field_privacy(&item_struct.fields, req);
            }
            Item::Enum(item_enum) if item_enum.ident == req.type_name => {
                type_found = true;
                // Enums have no single struct-level field to check; leave
                // field_is_private as None (not applicable) rather than guessing.
            }
            Item::Impl(item_impl) => {
                if impl_targets_type(item_impl, &req.type_name) {
                    for impl_item in &item_impl.items {
                        if let ImplItem::Fn(method) = impl_item {
                            methods_seen.push(method.sig.ident.to_string());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let methods_found = req
        .required_methods
        .iter()
        .map(|name| (name.clone(), methods_seen.iter().any(|seen| seen == name)))
        .collect();

    let mut forbidden_visitor = ForbiddenConstructionVisitor::new(&req.forbidden_constructions);
    forbidden_visitor.visit_file(&ast);
    let forbidden_constructions_absent = req
        .forbidden_constructions
        .iter()
        .map(|name| (name.clone(), !forbidden_visitor.found.contains(name)))
        .collect();

    Ok(ScanResult { type_found, field_is_private, methods_found, forbidden_constructions_absent })
}

/// Observe the privacy of the claimed field within `fields`.
///
/// For a tuple struct, the claimed field is matched positionally (its single field),
/// since tuple struct fields have no name of their own. For a named-field struct, the
/// claimed field is matched by `req.field_name`. Returns `None` if no matching field
/// is found.
fn observe_field_privacy(fields: &Fields, req: &RequiredStructure) -> Option<bool> {
    match fields {
        Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => {
            let field = unnamed.unnamed.first()?;
            Some(matches!(field.vis, Visibility::Inherited))
        }
        Fields::Named(named) => named.named.iter().find_map(|field| {
            field.ident.as_ref().and_then(|ident| {
                if *ident == req.field_name {
                    Some(matches!(field.vis, Visibility::Inherited))
                } else {
                    None
                }
            })
        }),
        _ => None,
    }
}

/// Whether an `impl` block's Self type is a bare path matching `type_name` (i.e.
/// `impl TypeName { .. }` or `impl Trait for TypeName { .. }`, not a qualified or
/// generic-parameterized path we can't syntactically resolve here).
fn impl_targets_type(item_impl: &syn::ItemImpl, type_name: &str) -> bool {
    if let syn::Type::Path(type_path) = &*item_impl.self_ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            return last_segment.ident == type_name;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Absolute path to the real bcinr checkout's `fixed.rs`, used as grounding
    /// evidence that this scanner works against real, non-fixture source. This test
    /// requires the bcinr checkout to exist at this path on disk; it is `#[ignore]`d
    /// so that `cargo test -p chicago-claims` stays hermetic by default. Run
    /// explicitly with `cargo test -p chicago-claims -- --ignored` when the bcinr
    /// checkout is present.
    const BCINR_FIXED_RS: &str = "/Users/sac/bcinr/crates/bcinr-cmca/src/fixed.rs";

    fn write_temp_rs(contents: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::Builder::new().suffix(".rs").tempfile().expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        f
    }

    fn numeric_fault_set_requirement() -> RequiredStructure {
        RequiredStructure {
            type_name: "NumericFaultSet".to_string(),
            field_name: "0".to_string(),
            field_must_be_private: true,
            required_methods: vec!["union".to_string()],
            forbidden_constructions: vec![],
        }
    }

    #[test]
    fn finds_private_tuple_field_and_method_in_fixture_source() {
        let source = r#"
            pub struct NumericFaultSet(u32);

            impl NumericFaultSet {
                pub const fn union(self, other: Self) -> Self {
                    Self(self.0 | other.0)
                }
            }
        "#;
        let file = write_temp_rs(source);

        let result = scan_required_structure(file.path(), &numeric_fault_set_requirement())
            .expect("scan should succeed on valid Rust source");

        assert!(result.type_found);
        assert_eq!(result.field_is_private, Some(true));
        assert_eq!(result.methods_found, vec![("union".to_string(), true)]);
    }

    #[test]
    fn reports_missing_type_without_panicking() {
        let source = "pub struct SomethingElse(u32);";
        let file = write_temp_rs(source);

        let result = scan_required_structure(file.path(), &numeric_fault_set_requirement())
            .expect("scan should succeed even when the type is absent");

        assert!(!result.type_found);
        assert_eq!(result.field_is_private, None);
        assert_eq!(result.methods_found, vec![("union".to_string(), false)]);
    }

    #[test]
    fn malformed_rust_source_yields_typed_error_not_panic() {
        let source = "pub struct Broken( ; ; ;";
        let file = write_temp_rs(source);

        let result = scan_required_structure(file.path(), &numeric_fault_set_requirement());

        match result {
            Err(ScanError::Parse { .. }) => {}
            other => panic!("expected ScanError::Parse, got {other:?}"),
        }
    }

    #[test]
    fn missing_file_yields_typed_io_error_not_panic() {
        let missing = Path::new("/nonexistent/chicago-claims/does-not-exist.rs");

        let result = scan_required_structure(missing, &numeric_fault_set_requirement());

        match result {
            Err(ScanError::Io { .. }) => {}
            other => panic!("expected ScanError::Io, got {other:?}"),
        }
    }

    /// Grounding evidence test: run the scanner against the REAL bcinr-cmca
    /// `fixed.rs` and confirm `NumericFaultSet` is observed as a struct with a
    /// private inner field and a `union` method. This is syntax-level evidence
    /// only (see module doc comment) — it does not prove `union` is semantically a
    /// join-semilattice operator, only that the AST shape claimed exists.
    #[test]
    #[ignore = "requires the bcinr checkout to exist at BCINR_FIXED_RS on disk"]
    fn real_bcinr_fixed_rs_has_numeric_fault_set_correctly_structured() {
        let path = Path::new(BCINR_FIXED_RS);
        assert!(
            path.exists(),
            "bcinr checkout not found at {BCINR_FIXED_RS}; this grounding test requires it"
        );

        let req = RequiredStructure {
            type_name: "NumericFaultSet".to_string(),
            field_name: "0".to_string(),
            field_must_be_private: true,
            required_methods: vec!["union".to_string()],
            forbidden_constructions: vec![],
        };

        let result = scan_required_structure(path, &req)
            .expect("real fixed.rs should parse as valid Rust source");

        assert!(result.type_found, "NumericFaultSet type not found in real fixed.rs");
        assert_eq!(
            result.field_is_private,
            Some(true),
            "NumericFaultSet's inner field is not observed as private"
        );
        assert_eq!(
            result.methods_found,
            vec![("union".to_string(), true)],
            "NumericFaultSet::union not found in an impl block"
        );
    }
}
