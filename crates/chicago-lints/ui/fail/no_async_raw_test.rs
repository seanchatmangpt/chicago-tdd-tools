// compile-flags: --crate-type lib --edition 2021
#![allow(dead_code)]

async fn fetch() -> u32 { 42 }

#[tokio::test]
async fn raw_async_test() {
    assert_eq!(fetch().await, 42, "should be 42");
}
