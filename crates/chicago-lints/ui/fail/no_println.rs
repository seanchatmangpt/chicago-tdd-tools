// compile-flags: --crate-type lib
#![allow(dead_code)]

pub fn process(id: u32) {
    println!("processing {id}");
    eprintln!("error: {id}");
}

#[cfg(test)]
mod tests {
    pub fn ok_in_test() {
        println!("debug output in test — should NOT warn");
    }
}
