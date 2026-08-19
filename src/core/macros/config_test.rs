//! Config Test Macros for star-toml-backed configuration (`DoD` ST-203)
//!
//! `examples/star-toml/GAP_REPORT.md` identified that every config test in
//! that example repeats the same three-line `TrustedLoader` setup:
//!
//! ```text
//! let loader = star_toml::trusted().layer_str(config_str, label);
//! let result = loader.load_admitted::<T>();
//! assert_ok!(&result);
//! ```
//!
//! `config_test!` and `config_refusal_test!` wrap that pattern in one
//! invocation, the same way [`crate::test!`] wraps the AAA `#[test]`
//! boilerplate. They call into the real `star_toml::trusted()` /
//! `load_admitted` collaborators (Chicago-style: no mock loader) — the
//! caller crate must depend on `star_toml` directly, since these are
//! `macro_rules!` macros expanded at the call site, not functions owned by
//! this crate.

/// Wrap a positive (admission-succeeds) `star_toml` config test in one invocation.
///
/// Builds a single-layer `TrustedLoader` from an inline TOML string, asserts
/// the load is admitted, and hands the admitted value to the body closure
/// for further assertions.
///
/// # Example
///
/// ```rust,ignore
/// use chicago_tdd_tools::config_test;
///
/// config_test!(test_default_config_admitted, AppConfig, r#"
///     name = "app"
///     workers = 4
/// "#, |config| {
///     assert_eq!(config.workers, 4);
/// });
/// ```
#[macro_export]
macro_rules! config_test {
    ($name:ident, $config_ty:ty, $config_str:expr, |$config:ident| $body:block) => {
        $crate::test!($name, {
            let __label: &'static str = concat!(stringify!($name), ".toml");
            let __result = star_toml::trusted()
                .layer_str($config_str, __label)
                .load_admitted::<$config_ty>();
            $crate::assert_ok!(&__result);
            #[allow(clippy::unwrap_used)]
            // JUSTIFICATION: assert_ok! above already proved __result is Ok.
            let __admitted = __result.unwrap();
            let $config = __admitted.value();
            $body
        });
    };
    ($name:ident, $config_ty:ty, $config_str:expr) => {
        $crate::config_test!($name, $config_ty, $config_str, |_config| {});
    };
}

/// Wrap a negative (admission-refused) `star_toml` config test in one invocation.
///
/// Builds a single-layer `TrustedLoader` from an inline TOML string and
/// asserts the load is refused. With the optional `|err|` form, hands the
/// error to the body closure for further assertions (e.g. a refusal code
/// check).
///
/// # Example
///
/// ```rust,ignore
/// use chicago_tdd_tools::config_refusal_test;
///
/// config_refusal_test!(test_invalid_port_refused, AppConfig, r#"
///     name = "app"
///     workers = 4
///     [server]
///     host = "localhost"
///     port = 0
/// "#);
/// ```
#[macro_export]
macro_rules! config_refusal_test {
    ($name:ident, $config_ty:ty, $config_str:expr, |$err:ident| $body:block) => {
        $crate::test!($name, {
            let __label: &'static str = concat!(stringify!($name), ".toml");
            let __result = star_toml::trusted()
                .layer_str($config_str, __label)
                .load_admitted::<$config_ty>();
            $crate::assert_err!(&__result);
            #[allow(clippy::unwrap_used)]
            // JUSTIFICATION: assert_err! above already proved __result is Err.
            let $err = __result.unwrap_err();
            $body
        });
    };
    ($name:ident, $config_ty:ty, $config_str:expr) => {
        $crate::config_refusal_test!($name, $config_ty, $config_str, |_err| {});
    };
}
