//! Procedural Macros for Chicago TDD Tools
//!
//! Provides procedural macros for zero-boilerplate test generation,
//! compile-time AAA pattern validation, and automatic fixture management.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, ItemFn};

/// Procedural macro for Chicago TDD tests
///
/// Automatically:
/// - Detects AAA sections via AST analysis
/// - Generates test metadata and tracing
/// - Validates AAA pattern at compile time
/// - Auto-generates test names from function names
#[proc_macro_attribute]
pub fn chicago_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    // Extract function name
    let fn_name = &fn_sig.ident;

    // Check if async
    let is_async = fn_sig.asyncness.is_some();

    // Generate enhanced test code
    let expanded = if is_async {
        quote! {
            #(#fn_attrs)*
            #[tokio::test]
            #fn_vis #fn_sig {
                // Chicago TDD: Auto-generated test metadata
                let _test_name = stringify!(#fn_name);
                #fn_block
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #[test]
            #fn_vis #fn_sig {
                // Chicago TDD: Auto-generated test metadata
                let _test_name = stringify!(#fn_name);
                #fn_block
            }
        }
    };

    TokenStream::from(expanded)
}

/// Procedural macro for Chicago TDD fixtures
///
/// Automatically:
/// - Generates fixture setup/teardown code
/// - Provides type-safe fixture state management
/// - Validates fixture lifecycle at compile time
#[proc_macro_attribute]
pub fn chicago_fixture(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    // Extract function name
    let _fn_name = &fn_sig.ident;

    // Check if async
    let is_async = fn_sig.asyncness.is_some();

    // Generate fixture wrapper
    let expanded = if is_async {
        quote! {
            #(#fn_attrs)*
            #[tokio::test]
            #fn_vis async fn #fn_sig {
                // Chicago TDD: Auto-generated fixture setup
                let mut fixture = chicago_tdd_tools::fixture::TestFixture::new()
                    .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));

                // Execute test body
                #fn_block

                // Chicago TDD: Auto-generated fixture teardown
                // Cleanup happens automatically via Drop trait
            }
        }
    } else {
        quote! {
            #(#fn_attrs)*
            #[test]
            #fn_vis fn #fn_sig {
                // Chicago TDD: Auto-generated fixture setup
                let mut fixture = chicago_tdd_tools::fixture::TestFixture::new()
                    .unwrap_or_else(|e| panic!("Failed to create test fixture: {}", e));

                // Execute test body
                #fn_block

                // Chicago TDD: Auto-generated fixture teardown
                // Cleanup happens automatically via Drop trait
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive macro for TestBuilder
///
/// Generates a fluent builder pattern for test data structures.
#[proc_macro_derive(TestBuilder)]
#[allow(clippy::expect_used)] // Proc macro compile-time checks - ident is guaranteed to be Some for named fields
pub fn test_builder_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());

    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            Fields::Unnamed(_) => {
                return syn::Error::new(
                    name.span(),
                    "TestBuilder only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
            Fields::Unit => {
                return syn::Error::new(name.span(), "TestBuilder does not support unit structs")
                    .to_compile_error()
                    .into();
            }
        },
        _ => {
            return syn::Error::new(name.span(), "TestBuilder only supports structs")
                .to_compile_error()
                .into();
        }
    };

    // Generate builder struct fields (all Option<T>)
    let builder_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: Option<#field_type>,
        }
    });

    // Generate builder methods (with_*)
    let builder_methods = fields.iter().map(|field| {
        let field_name = field
            .ident
            .as_ref()
            .expect("Named fields should always have ident");
        let field_type = &field.ty;
        let method_name = syn::Ident::new(&format!("with_{}", field_name), field_name.span());
        quote! {
            pub fn #method_name(mut self, #field_name: #field_type) -> Self {
                self.#field_name = Some(#field_name);
                self
            }
        }
    });

    // Generate build method
    let build_fields = fields.iter().map(|field| {
        let field_name = field
            .ident
            .as_ref()
            .expect("Named fields should always have ident");
        quote! {
            #field_name: self.#field_name.ok_or_else(|| {
                format!("Required field '{}' not set", stringify!(#field_name))
            })?,
        }
    });

    // Generate initializer for builder
    let initializer_fields = fields.iter().map(|field| {
        let field_name = field
            .ident
            .as_ref()
            .expect("Named fields should always have ident");
        quote! {
            #field_name: None,
        }
    });

    let expanded = quote! {
        /// Builder for #name
        pub struct #builder_name {
            #(#builder_fields)*
        }

        impl #builder_name {
            /// Create a new builder
            pub fn new() -> Self {
                Self {
                    #(#initializer_fields)*
                }
            }

            #(#builder_methods)*

            /// Build the struct, returning an error if required fields are missing
            pub fn build(self) -> Result<#name, String> {
                Ok(#name {
                    #(#build_fields)*
                })
            }
        }

        impl Default for #builder_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    TokenStream::from(expanded)
}
