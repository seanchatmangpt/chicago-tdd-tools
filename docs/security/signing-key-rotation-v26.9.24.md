# Signing-key rotation (v26.9.24)

Recorded 2026-09-24 (fleet key scan after the single-repo migration). Base `a34435145121` of `chicago-tdd-tools`.
Every private key listed here was committed to this repository and is therefore compromised: every receipt or
attestation signed with it carries no signing authority (standing REFUSED, broken_term R_missing_authority).
The keys leave the tree (history is not rewritten; no force-push), and each key directory's `.gitignore` now
covers both halves. Every checkout keeps its own pair: ggen generates one on first use, and a tracked public
half without its private half would make that first `ggen sync` refuse [FM-KEY-010/011]. The canonical
checkout's new public key is published below for anyone verifying its future receipts.

| key dir | removed private key sha256 | removed public key sha256 | new public key (canonical checkout) |
|---|---|---|---|
| `.ggen/keys` | `3b0d42100fd41d181f4f702697ba5438c0515ae408bce1fa2847a41b9a5a2a79` | `b710ddf4253d73e2e96d5678aef83f76a2eb1afd80d617dff1bf43bb5dd5791a` | `b329dee19a8d60c8984796cb5187494e41610b10f1f0a0561f35a3df388282a3` |
