# Release Notes — chicago-tdd-tools v26.10.12 (2026-10-12)

> Release notes for workspace version 26.10.12 (`release-validate-version` green).

## Headline

Typed-error hardening across the conformance, ledger, and nato surfaces: failures surface
as typed, matchable errors instead of stringly channels, keeping the fail-closed contract
intact from POWL loading through ledger sealing and NATO court receipts. This train also
closes the dependency security advisory set. A hardening and hygiene release — no new
capabilities.

## Hardened

- `conformance`: loader/session/validate failures typed (StateMachineError, SealError,
  GovernanceError); JSON-path activity resolution enforced at validate time; 4-step
  classification law documented.
- `ledger`/`nato`: DossierWireError, SwarmError, ReceiptJsonError, BuilderError,
  PipelineError, GovernanceError, WeaverError, OtelError — every public fallible surface
  typed; crate-root + prelude re-exports.
- Guest ABI v2 substrate: guest-abi-v1 shim (probe→1, 34-byte legacy seal preimage, errno
  remap 14/15/16→11), CTT_MODULE_DIGEST slot + ctt-splice-digest tool.

## Security

- Dependency advisory closure: quick-xml 0.41 (via wasm4pm-compat 26.8.7), h2 0.4.20,
  rustls 0.23.45, git2 0.21, dead reqwest 0.11 removed, serde_yaml → serde_yaml_ng.
  Remaining audit warnings (unsafe-libyaml via serde_yaml_ng's own driver; chacha20
  yanked) are disclosed and not lock-fixable — see the findings ledger.

## Notes

- This file intentionally does NOT duplicate `docs/releases/CHANGELOG.md` (`[26.10.12]`
  entry) — see that for the full per-commit ledger and wave addenda.
