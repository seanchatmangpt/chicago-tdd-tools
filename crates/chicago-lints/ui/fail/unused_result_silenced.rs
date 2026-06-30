// compile-flags: --crate-type lib
#![allow(dead_code)]

use std::num::ParseIntError;

fn fallible() -> Result<i32, ParseIntError> {
    "42".parse()
}

fn maybe() -> Option<i32> {
    Some(42)
}

pub fn bad() {
    let _ = fallible();
    let _ = maybe();
}

pub fn good() -> Result<(), ParseIntError> {
    fallible()?;
    let _val = fallible()?;
    Ok(())
}
