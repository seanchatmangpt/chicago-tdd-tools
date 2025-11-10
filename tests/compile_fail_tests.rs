#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/validated_run_compile_error.rs");
    t.compile_fail("tests/compile-fail/validated_batch_compile_error.rs");
}
