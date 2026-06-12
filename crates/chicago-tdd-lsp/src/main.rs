//! chicago-tdd-lsp — dev-dependency guard
//!
// Binary entrypoint: unwrap/expect on fatal I/O at startup is the correct panic boundary.
#![allow(clippy::unwrap_used)]
//! Watches every `Cargo.toml` opened in the editor and emits a diagnostic
//! (CTDD-DEV-001) whenever `chicago-tdd-tools` appears in `[dependencies]`
//! instead of `[dev-dependencies]`.
//!
//! Built on lsp-max — the law-state LSP runtime. Never uses lsp-server.
//!
//! ## Features
//!
//! | Flag | Adds |
//! |------|------|
//! | *(default)* | Diagnostics only: CTDD-DEV-001 errors on open/change/save |
//! | `extended` | + `textDocument/hover` (law text + fix guidance over flagged range) |
//! |             | + `textDocument/codeAction` (quickfix: move to [dev-dependencies]) |
//! |             | + structured `data` field: `conformance_vector`, `receipt_obligation` |

mod check;

use async_trait::async_trait;
use check::find_violations;
#[cfg(feature = "extended")]
use lsp_max::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse, Hover,
    HoverContents, HoverParams, MarkedString, WorkspaceEdit,
};
use lsp_max::{
    jsonrpc::Result,
    lsp_types::{
        Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
        DidOpenTextDocumentParams, DidSaveTextDocumentParams, InitializeParams, InitializeResult,
        Position, Range, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
    },
    Client, LanguageServer, LspService, Server,
};
use parking_lot::RwLock;
use std::collections::HashMap;

// ── Backend ───────────────────────────────────────────────────────────────────

struct Backend {
    client: Client,
    /// uri → (text, violations at last check). Read-heavy: written on open/change, read on hover/code_action.
    docs: RwLock<HashMap<Url, (String, Vec<check::Violation>)>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self { client, docs: RwLock::new(HashMap::new()) }
    }

    async fn recheck(&self, uri: Url, text: String) {
        let violations = find_violations(&text);
        let diags: Vec<Diagnostic> = violations.iter().map(to_diagnostic).collect();
        self.docs.write().insert(uri.clone(), (text, violations));
        self.client.publish_diagnostics(uri, diags, None).await;
    }
}

fn to_diagnostic(v: &check::Violation) -> Diagnostic {
    let range = Range {
        start: Position { line: v.line, character: v.char_start },
        end: Position { line: v.line, character: v.char_end },
    };
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::ERROR),
        code: Some(lsp_max::lsp_types::NumberOrString::String(v.code.to_string())),
        source: Some("chicago-tdd-lsp".to_string()),
        message: v.message.to_string(),
        #[cfg(feature = "extended")]
        data: Some(serde_json::json!({
            "law": v.law_text,
            "fix": v.fix_guidance,
            "receipt_obligation": "CTDD-DEV-001-receipt",
            "conformance_vector": {
                "admitted": [],
                "refused": ["dev-dependency-law"],
                "unknown": []
            }
        })),
        #[cfg(not(feature = "extended"))]
        data: None,
        related_information: None,
        tags: None,
        code_description: None,
    }
}

// ── LanguageServer impl ───────────────────────────────────────────────────────

#[async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                #[cfg(feature = "extended")]
                hover_provider: Some(lsp_max::lsp_types::HoverProviderCapability::Simple(true)),
                #[cfg(feature = "extended")]
                code_action_provider: Some(
                    lsp_max::lsp_types::CodeActionProviderCapability::Simple(true),
                ),
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        if is_cargo_toml(&uri) {
            self.recheck(uri, params.text_document.text).await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if is_cargo_toml(&uri) {
            if let Some(change) = params.content_changes.into_iter().last() {
                self.recheck(uri, change.text).await;
            }
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        if is_cargo_toml(&uri) {
            let text = params.text.unwrap_or_default(); // Option<String> from LSP — default to empty on None
            self.recheck(uri, text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.docs.write().remove(&uri);
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    // ── extended feature: hover ───────────────────────────────────────────────

    #[cfg(feature = "extended")]
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;
        let docs = self.docs.read();
        let hover = docs.get(uri).and_then(|(_, viols)| {
            viols.iter().find(|v| {
                v.line == pos.line && pos.character >= v.char_start && pos.character <= v.char_end
            })
        });
        Ok(hover.map(|v| Hover {
            contents: HoverContents::Scalar(MarkedString::String(format!(
                "**{}** `chicago-tdd-tools`\n\n{}\n\nFix: {}",
                v.code, v.law_text, v.fix_guidance
            ))),
            range: Some(Range {
                start: Position { line: v.line, character: v.char_start },
                end: Position { line: v.line, character: v.char_end },
            }),
        }))
    }

    // ── extended feature: code action ─────────────────────────────────────────

    #[cfg(feature = "extended")]
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let req_range = params.range;
        let docs = self.docs.read();
        let actions: Vec<CodeActionOrCommand> = docs
            .get(uri)
            .map(|(_, viols)| {
                viols
                    .iter()
                    .filter(|v| {
                        let vr = Range {
                            start: Position { line: v.line, character: v.char_start },
                            end: Position { line: v.line, character: v.char_end },
                        };
                        ranges_overlap(vr, req_range)
                    })
                    .map(|v| {
                        CodeActionOrCommand::CodeAction(CodeAction {
                            title: "Move `chicago-tdd-tools` to [dev-dependencies]".to_string(),
                            kind: Some(CodeActionKind::QUICKFIX),
                            diagnostics: Some(vec![to_diagnostic(v)]),
                            edit: Some(WorkspaceEdit::default()),
                            is_preferred: Some(true),
                            ..Default::default()
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(if actions.is_empty() { None } else { Some(actions) })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn is_cargo_toml(uri: &Url) -> bool {
    uri.path().as_str().ends_with("Cargo.toml")
}

#[cfg(feature = "extended")]
fn ranges_overlap(a: Range, b: Range) -> bool {
    a.start <= b.end && b.start <= a.end
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket)
        .serve(service)
        .await
        .expect("LSP server exited with a fatal I/O error");
}
