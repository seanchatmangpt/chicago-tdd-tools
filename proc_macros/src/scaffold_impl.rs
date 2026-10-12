// proc-macro code runs at compile time; unwrap_or/unwrap_or_else are correct fallback patterns here
#![allow(clippy::unwrap_used)]
// AUDIT NOTE: proc-macro stdout is NOT parsed for cargo:: directives (that is
// a build-script-only protocol) — see the disclosure in scaffold_impl below.

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Result, Token,
};

use crate::path_resolver::{extract_ticket_id, workspace_root};

struct ScaffoldArgs {
    ticket_lit: LitStr,
    test_lit: LitStr,
}

impl Parse for ScaffoldArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse: ticket = "...", test = "..."
        let ticket_key: syn::Ident = input.parse()?;
        if ticket_key != "ticket" {
            return Err(syn::Error::new(ticket_key.span(), "expected `ticket = \"...\"`"));
        }
        let _eq: Token![=] = input.parse()?;
        let ticket_lit: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;

        let test_key: syn::Ident = input.parse()?;
        if test_key != "test" {
            return Err(syn::Error::new(test_key.span(), "expected `test = \"...\"`"));
        }
        let _eq2: Token![=] = input.parse()?;
        let test_lit: LitStr = input.parse()?;
        // optional trailing comma
        let _: Option<Token![,]> = input.parse()?;

        Ok(Self { ticket_lit, test_lit })
    }
}

pub fn scaffold_impl(input: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(input as ScaffoldArgs);

    let ticket_val = args.ticket_lit.value();
    let test_val = args.test_lit.value();

    // Resolve workspace root from CARGO_MANIFEST_DIR
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = std::path::Path::new(&manifest_dir);

    let root = workspace_root(manifest_path).unwrap_or_else(|| manifest_path.to_path_buf());

    let ticket_path = root.join(&ticket_val);
    let test_path = root.join(&test_val);

    // Validate ticket file exists
    if !ticket_path.exists() {
        let msg = format!(
            "chicago-tdd-tools: ticket file not found at workspace path\n  \"{}\"            \n  Resolved to: {}\n  Create the file or correct the path.",
            ticket_val,
            ticket_path.display()
        );
        let err = syn::Error::new(args.ticket_lit.span(), msg);
        return err.into_compile_error().into();
    }

    // Validate test file exists
    if !test_path.exists() {
        let msg = format!(
            "chicago-tdd-tools: test file not found at workspace path\n  \"{}\"            \n  Resolved to: {}\n  Create the file or correct the path.",
            test_val,
            test_path.display()
        );
        let err = syn::Error::new(args.test_lit.span(), msg);
        return err.into_compile_error().into();
    }

    // AUDIT DISCLOSURE (wave-8 re-apply): the ticket gate fires at macro
    // expansion. After a ticket is deleted, only a clean rebuild re-fires it
    // (proc-macro output is cached) — disclosed, not mechanized.
    // `proc_macro::tracked_path` is the nightly-only mechanized alternative.

    // Extract ticket ID for the warning message
    let ticket_id = extract_ticket_id(&ticket_val).unwrap_or_else(|| ticket_val.clone());

    // Runtime pending marker: the expansion panics via scaffold_pending; W27
    // probe correction: catch_scaffold absorbs the SCAFFOLD PENDING panic as a
    // CANDIDATE pass — a missing ticket makes the test PASS-as-CANDIDATE (with
    // an eprintln warning), not fail.

    let ticket_id_lit = LitStr::new(&ticket_id, Span::call_site());
    let ticket_lit = args.ticket_lit;
    let test_lit = args.test_lit;

    quote! {
        ::chicago_tdd_tools::__runtime::scaffold_pending(
            #ticket_id_lit,
            #ticket_lit,
            #test_lit,
        )
    }
    .into()
}
