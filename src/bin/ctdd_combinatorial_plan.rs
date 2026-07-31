//! Read-only CLI projection of the generated combinatorial-maximalism kernel.

#[allow(dead_code, missing_docs, unused_imports)]
#[path = "../../architecture/combinatorial-maximalism/generated/src/lib.rs"]
mod cmd;

use cmd::{compose, CompositionRequest, PROFILES};
use std::process::ExitCode;

fn print_usage() {
    eprintln!("usage: ctdd_combinatorial_plan --list | <profile-id> [--external]");
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(first) = args.next() else {
        print_usage();
        return ExitCode::from(2);
    };

    if first == "--list" {
        for profile in PROFILES {
            println!("{}\t{}\texternal={}\tinclude_all={}", profile.id, profile.title, profile.external_allowed, profile.include_all);
        }
        return ExitCode::SUCCESS;
    }

    let include_external = args.any(|argument| argument == "--external");
    match compose(CompositionRequest { profile: &first, include_external }) {
        Ok(plan) => {
            println!("profile={}", plan.profile.id);
            println!("standing={:?}", plan.standing);
            println!("realizations={}", plan.realizations.len());
            println!("projections={}", plan.projections.len());
            println!("external_contracts={}", plan.external_contracts.len());
            println!("fingerprint={:016x}", plan.fingerprint);
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("refusal={} detail={refusal:?}", refusal.code());
            ExitCode::from(2)
        }
    }
}
