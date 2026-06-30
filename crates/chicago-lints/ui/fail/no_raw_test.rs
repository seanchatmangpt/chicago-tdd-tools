// compile-flags: --crate-type lib
#![allow(dead_code)]

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn raw_test_no_wrapper() {
    assert_eq!(add(1, 2), 3);
}
