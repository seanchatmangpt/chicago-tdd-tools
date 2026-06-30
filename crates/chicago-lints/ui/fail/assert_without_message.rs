// compile-flags: --crate-type lib
#![allow(dead_code)]

#[cfg(test)]
mod tests {
    fn add(a: i32, b: i32) -> i32 { a + b }

    #[test]
    fn missing_message() {
        assert_eq!(add(1, 2), 3);
        assert_ne!(add(1, 2), 0);
        assert!(add(1, 2) > 0);
    }

    #[test]
    fn has_message_ok() {
        assert_eq!(add(1, 2), 3, "1+2 should be 3");
        assert_ne!(add(1, 2), 0, "result should not be zero");
        assert!(add(1, 2) > 0, "result should be positive");
    }
}
