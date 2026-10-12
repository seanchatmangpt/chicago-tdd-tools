# Tutorial: Build and Seal a STANAG 4107 Dossier
> **FRESHNESS NOTE (2026-10-12):** the dossier was re-sealed after this page was
> written — the expected merkle root below is stale; the live root is in
> docs/nato/NATO_STANAG_4107_DOSSIER.json (`just dossier-verify` recomputes it).

Seal twenty NATO court receipts into a signed `CTT-DOSSIER-v1` document, then verify the ML-DSA-65 signature and Merkle root. Every command below was executed against the current tree on 2026-10-11; the expected Merkle root `783308d9…6671e` is the value sealed in the landed dossier.

Prerequisites: both checkouts at `/Users/sac/chicago-tdd-tools` and `/Users/sac/affidavit`; `cargo` on PATH.

## 1. Run the courts and emit receipts

```console
$ cd /Users/sac/chicago-tdd-tools
$ just emit-dossier
```

This recipe (Justfile lines 222-224) does two things in order:

1. `CTT_EMIT_RECEIPTS=1 timeout 300s cargo test --release --lib --features nato,ledger -- nato::dossier_court --nocapture` — runs the court suite. The opt-in test `emit_dossier_receipts_when_opted_in` (src/nato/dossier_court.rs:427) writes the twenty receipts to `docs/receipts/N01.json` … `N20.json` because `CTT_EMIT_RECEIPTS=1`. Without the env var the default test run stays hermetic.
2. `cargo run --manifest-path /Users/sac/affidavit/Cargo.toml --features crypto-trust --bin affi -- dossier emit --receipts-dir docs/receipts --out docs/nato/NATO_STANAG_4107_DOSSIER.json --format json`

Fail-closed gates on the emit path (`/Users/sac/affidavit/src/handlers.rs`, `dossier_emit`):

- The receipts directory must exist and be a directory, or a typed `Execution` error names it.
- Exactly twenty receipts are required: `expected 20 court receipts, found N`.
- Every receipt must be `Verdict::Pass` **and** `ControlResult::MutantCaught`; the refusal lists the failing courts: `dossier admission refused: courts without Pass+MutantCaught: N05, N11`.
- The signing key is loaded through `load_ctt_signer()` → `KeyStore::new().load_or_create()` (`.ggen/keys/ledger/ml_dsa_65.seed`, generated on first use).

Expected success output ends with:

```json
{
  "ok": true,
  "cmd": "dossier emit",
  "receipts": 20,
  "merkle_root": "783308d923d5fd308c93df83cc903cf7745921035bf928f0c72420097226671e",
  "out": "docs/nato/NATO_STANAG_4107_DOSSIER.json"
}
```

## 2. Verify the seal

```console
$ just dossier-verify
```

which runs:

```console
$ cargo run --manifest-path /Users/sac/affidavit/Cargo.toml --features crypto-trust --bin affi -- dossier inspect docs/nato/NATO_STANAG_4107_DOSSIER.json --verify-pqc-signature ML-DSA-65 --assert-merkle-root true --format json
```

Expected output (this exact run exited 0 on 2026-10-11):

```json
{
  "assert_merkle_root": true,
  "cmd": "dossier inspect",
  "format": "CTT-DOSSIER-v1",
  "merkle_root": "783308d923d5fd308c93df83cc903cf7745921035bf928f0c72420097226671e",
  "ok": true,
  "receipts": 20,
  "signature_verified": true
}
```

What inspect checks, in order (handlers.rs `dossier_inspect`, fail-closed):

1. `format` must equal `CTT-DOSSIER-v1`; anything else is refused.
2. Exactly 20 embedded receipts decode through the domain `CourtReceipt::from_json`.
3. The Merkle root is recomputed with `dossier_root(&receipts)` and must equal the stored `merkle_root`.
4. `--verify-pqc-signature ML-DSA-65` (the only implemented algorithm; any other value is a typed refusal) verifies the 3309-byte signature over `dossier_subject(root)` under the KeyStore public key, after the 16-hex-char `key_id` matches.

## 3. Inspect an arbitrary dossier file

```console
$ just dossier-inspect docs/nato/NATO_STANAG_4107_DOSSIER.json
```

## 4. What just happened

- The 20 court receipts hash into a 32-byte Merkle root via `dossier_root`.
- The root is framed by the domain tag `chicago-ledger-dossier-subject-v1` (`dossier_subject`, src/ledger/signing/subject.rs: `BLAKE3(u32le(len) || domain || root)`) and signed with ML-DSA-65 using fixed context `chicago-ledger-dossier-v1`.
- The dossier JSON carries `format`, `court_receipts`, `merkle_root`, `key_id`, `signature` (6618 lowercase hex chars = 3309 bytes), `sealed_at`.
- The output file is written atomically (tmp + fsync + rename, `write_output` in src/ledger/dossier_wire.rs), so a crash never leaves a partial dossier at the real path.

## Troubleshooting

- `REFUSED_UNSUPPORTED: the cryptographic trust plane is not compiled into this binary` — the affi binary was built without `--features crypto-trust`. Rebuild exactly as the Justfile does.
- `merkle root mismatch: stored … recomputed …` — the receipts under `docs/receipts/` changed after sealing. Re-run `just emit-dossier`.
- `ML-DSA-65 signature verification failed` — the dossier was sealed under a different seed than the current `.ggen/keys/ledger/ml_dsa_65.seed`. Re-seal with the current key.
