// compile-flags: --crate-type lib
#![allow(dead_code, unused_variables)]

use std::num::ParseIntError;

fn fallible() -> Result<i32, ParseIntError> {
    "42".parse()
}

// Named binding with propagation: fine
pub fn propagate() -> Result<(), ParseIntError> {
    let _val = fallible()?;
    Ok(())
}

// Discarding a non-Result: fine
pub fn discard_plain() {
    let _ = 42_i32;
    let _ = "hello";
}

// println in cfg(test) module: fine
#[cfg(test)]
mod tests {
    pub fn debug_print() {
        println!("test debug output: ok");
    }
}
