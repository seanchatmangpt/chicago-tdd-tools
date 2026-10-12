# deontic — LKIF-Core Deontic Roles

Hand-authored LKIF-core vocabulary fixtures assigning deontic roles to the
`ex:AcmeCorp` / `ex:BetaHoldings` / `ex:MergerSub` merger scenario. These
files are consumed by the courts under `src/nato/` and are referenced from the
parent [README.md](../README.md).

## Tree

```
deontic/
├── merger-obligations.ttl   Obligation / Permission assignments
├── closing-conditions.ttl   precedent conditions to Closing
└── breach.ttl               Violation / Remedy roles
```

## Files

| File | Contents |
|------|----------|
| `merger-obligations.ttl` | Obligations plus `ex:PermissionOrdinaryCourse` (`lkif:Permission`) |
| `closing-conditions.ttl` | `ex:ClosingCondition` instances, `ex:requires` conjunction |
| `breach.ttl` | `lkif:Violation` / remedy roles; each violation carries `ex:excused false` |

## Limitations

The boolean encodings in these files (`ex:occurred`, `ex:excused false`, the
unpredicated ordinary-course Permission) flatten Hartian open-texture legal
standards into fixed bits. This fabric resolves contested conditions; it has
no abstention path. See the parent README's
[Open-Texture Limitations](../README.md#open-texture-limitations) section for
the full analysis.

## See Also

- `../README.md` — merger_matrix fixture set overview
- LKIF-core: https://www.estrellaproject.org/lkif-core/
