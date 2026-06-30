// compile-flags: --crate-type lib --edition 2021
#![allow(dead_code)]
#![feature(decl_macro)]

// Stub the log crate using decl_macro so `log::warn!` etc. resolve as path macros.
mod log {
    pub macro warn($($t:tt)*) { () }
    pub macro error($($t:tt)*) { () }
    pub macro info($($t:tt)*) { () }
    pub macro debug($($t:tt)*) { () }
    pub macro trace($($t:tt)*) { () }
}

pub fn process(attempt: u32) {
    log::warn!("retrying: {}", attempt);
    log::error!("failed: {}", attempt);
    log::info!("ok: {}", attempt);
    log::debug!("state: {}", attempt);
    log::trace!("trace: {}", attempt);
}

#[cfg(test)]
mod tests {
    pub fn debug_log() {
        log::debug!("test debug — ok");
    }
}
