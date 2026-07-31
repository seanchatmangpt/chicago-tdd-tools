//! Read-only CLI projection of the generated combinatorial-maximalism kernel.

#[allow(
    dead_code,
    missing_docs,
    unused_imports,
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic
)]
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
            let id = profile.id;
            let title = profile.title;
            let external = profile.external_allowed;
            let include_all = profile.include_all;
            println!("{id}\t{title}\texternal={external}\tinclude_all={include_all}");
        }
        return ExitCode::SUCCESS;
    }

    let include_external = args.any(|argument| argument == "--external");
    match compose(CompositionRequest { profile: &first, include_external }) {
        Ok(plan) => {
            let profile = plan.profile.id;
            let standing = plan.standing;
            let realizations = plan.realizations.len();
            let projections = plan.projections.len();
            let external_contracts = plan.external_contracts.len();
            let fingerprint = plan.fingerprint;
            println!("profile={profile}");
            println!("standing={standing:?}");
            println!("realizations={realizations}");
            println!("projections={projections}");
            println!("external_contracts={external_contracts}");
            println!("fingerprint={fingerprint:016x}");
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            let code = refusal.code();
            eprintln!("refusal={code} detail={refusal:?}");
            ExitCode::from(2)
        }
    }
}
