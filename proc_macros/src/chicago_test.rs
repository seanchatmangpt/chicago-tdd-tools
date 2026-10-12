// proc-macro code runs at compile time; unwrap_or/unwrap_or_else are correct fallback patterns here
#![allow(clippy::unwrap_used)]
// AUDIT DISCLOSURE: the ticket gate fires at macro expansion; after a ticket
// is deleted only a clean rebuild re-fires it (proc-macro output is cached).
// on stdout — the equivalent of a build script's println! protocol, not a production log line.
#![allow(clippy::print_stdout)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, ItemFn, LitStr, Result, Token,
};

use crate::path_resolver::{extract_ticket_id, workspace_root};

struct ChicagoTestArgs {
    ticket_lit: LitStr,
    scaffold_fn_lit: LitStr,
}

impl Parse for ChicagoTestArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let ticket_key: Ident = input.parse()?;
        if ticket_key != "ticket" {
            return Err(syn::Error::new(ticket_key.span(), "expected `ticket = \"...\"`"));
        }
        let _eq: Token![=] = input.parse()?;
        let ticket_lit: LitStr = input.parse()?;
        let _comma: Token![,] = input.parse()?;

        let sfn_key: Ident = input.parse()?;
        if sfn_key != "scaffold_fn" {
            return Err(syn::Error::new(sfn_key.span(), "expected `scaffold_fn = \"...\"`"));
        }
        let _eq2: Token![=] = input.parse()?;
        let scaffold_fn_lit: LitStr = input.parse()?;
        let _: Option<Token![,]> = input.parse()?;

        Ok(Self { ticket_lit, scaffold_fn_lit })
    }
}

pub fn chicago_test_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as ChicagoTestArgs);
    let func = syn::parse_macro_input!(item as ItemFn);

    let ticket_val = args.ticket_lit.value();
    let scaffold_fn_val = args.scaffold_fn_lit.value();

    // Audit W8 (re-applied after the wave-7 checkout incident): an async or
    // parameterized fn silently mis-expanded under the plain #[test] wrapper;
    // refuse with a spanned error instead.
    if func.sig.asyncness.is_some() {
        let err =
            syn::Error::new(func.sig.ident.span(), "chicago_test! does not support async test fns");
        return err.into_compile_error().into();
    }
    if !func.sig.inputs.is_empty() {
        let err = syn::Error::new(
            func.sig.ident.span(),
            "chicago_test! does not support parameterized test fns",
        );
        return err.into_compile_error().into();
    }

    // Resolve workspace root and validate ticket exists
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let manifest_path = std::path::Path::new(&manifest_dir);
    let root = workspace_root(manifest_path).unwrap_or_else(|| manifest_path.to_path_buf());
    let ticket_path = root.join(&ticket_val);

    if !ticket_path.exists() {
        let msg = format!(
            "chicago-tdd-tools: ticket file not found at workspace path\n  \"{}\"            \n  Resolved to: {}",
            ticket_val,
            ticket_path.display()
        );
        let err = syn::Error::new(args.ticket_lit.span(), msg);
        return err.into_compile_error().into();
    }

    // Derive test name: chicago_cc_NNN_<original>
    let ticket_id = extract_ticket_id(&ticket_val).unwrap_or_else(|| "unknown".to_string());
    let ticket_slug = ticket_id.to_lowercase().replace('-', "_");
    let original_name = func.sig.ident.to_string();
    let test_name_str = format!("chicago_{ticket_slug}_{original_name}");
    let test_name = Ident::new(&test_name_str, func.sig.ident.span());

    let ticket_id_lit = LitStr::new(&ticket_id, Span::call_site());
    let scaffold_fn_lit = args.scaffold_fn_lit;

    // suppress unused variable warning for scaffold_fn_val
    let _ = scaffold_fn_val;

    let body = &func.block;
    let attrs = &func.attrs;
    let vis = &func.vis;

    quote! {
        #[test]
        #(#attrs)*
        #vis fn #test_name() {
            ::chicago_tdd_tools::__runtime::catch_scaffold(
                #ticket_id_lit,
                #scaffold_fn_lit,
                || { #body },
            );
        }
    }
    .into()
}
