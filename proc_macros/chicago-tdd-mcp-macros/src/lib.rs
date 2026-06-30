//! Proc-macro attributes for chicago-tdd-mcp MCP testing.
//!
//! # `#[mcp_server_test]`
//!
//! Wraps an async test function with MCP server harness setup and teardown.
//! The test function receives a `McpSession<Ready>` as its only argument.
//!
//! ```ignore
//! use chicago_tdd_mcp_macros::mcp_server_test;
//!
//! #[mcp_server_test(bin = "oclnr-mcp")]
//! async fn lists_tools(mut session: chicago_tdd_mcp::McpSession<chicago_tdd_mcp::Ready>) {
//!     let tools = session.tools_list().await.unwrap();
//!     assert!(!tools.is_empty());
//! }
//! ```
//!
//! Binary resolution order:
//! 1. `OCLNR_MCP_BIN` env var (or the env var named by `env = "..."` argument)
//! 2. `which` lookup by binary name
//! 3. Test is skipped (not failed) if neither resolves

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, ItemFn, LitStr, Token,
};

struct TestArgs {
    bin: String,
    env_var: Option<String>,
}

impl Parse for TestArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut bin = String::new();
        let mut env_var = None;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            let val: LitStr = input.parse()?;
            match key.to_string().as_str() {
                "bin" => bin = val.value(),
                "env" => env_var = Some(val.value()),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown argument `{other}`; expected `bin` or `env`"),
                    ))
                }
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if bin.is_empty() {
            return Err(input.error("missing required argument `bin = \"...\"`"));
        }

        Ok(Self { bin, env_var })
    }
}

/// Attribute macro that wires MCP server harness setup/teardown around a test.
///
/// The decorated async function must accept exactly one argument of type
/// `chicago_tdd_mcp::McpSession<chicago_tdd_mcp::Ready>`.
#[proc_macro_attribute]
pub fn mcp_server_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as TestArgs);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_body = &func.block;
    let bin_name = &args.bin;

    // Env var name: explicit `env = "..."` or uppercase bin name + _BIN
    let env_var = args
        .env_var
        .unwrap_or_else(|| format!("{}_BIN", bin_name.to_uppercase().replace('-', "_")));

    let expanded = quote! {
        #[tokio::test]
        async fn #fn_name() {
            // Resolve binary: env var first, then PATH lookup
            let bin_path: Option<String> = std::env::var(#env_var).ok().or_else(|| {
                which::which(#bin_name).ok().map(|p| p.display().to_string())
            });

            let bin_path = match bin_path {
                Some(p) => p,
                None => {
                    eprintln!(
                        "Skipping {}: binary `{}` not found (set {} or put it in PATH)",
                        stringify!(#fn_name), #bin_name, #env_var
                    );
                    return;
                }
            };

            let mut cmd = tokio::process::Command::new(&bin_path);
            let harness = chicago_tdd_mcp::McpServerHarnessBuilder::new(cmd)
                .spawn()
                .await
                .expect("failed to spawn MCP server");

            let mut session = chicago_tdd_mcp::McpSession::new(harness)
                .initialize()
                .await
                .expect("MCP initialize handshake failed");

            let run = async move {
                let session = &mut session;
                #fn_body
            };
            run.await;
        }
    };

    expanded.into()
}
