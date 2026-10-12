# Combinatorial Maximalism

The canonical executable guide and generated registries live under [`architecture/combinatorial-maximalism/`](../../architecture/combinatorial-maximalism/).

The source of truth is `ontology.ttl`, `O.star.toml`, `ggen.toml`, and `templates/`. The `generated/` directory is a product and must never be hand-edited.

The read-only CLI adapter is:

```bash
# NOTE: generated-projection runner (no just recipe); run via cargo:
cargo run --bin ctdd_combinatorial_plan -- --list
cargo run --bin ctdd_combinatorial_plan -- profile.core-local
cargo run --bin ctdd_combinatorial_plan -- profile.external-integration --external
```

The CLI manufactures candidate plans only. It has no broker, provider, filesystem, process, network, credential, or deployment authority.
