# Reference: POWL Compact DSL Grammar and AshGraphLaw Capability API

> Diataxis axis: **Reference** (information-oriented). Transcribed from the
> parser at /Users/sac/chicago-tdd-tools/src/conformance/loader.rs and the
> generated capability modules in /Users/sac/ash_graphlaw.

## Part 1: POWL compact DSL

### Lexical structure (scanner, loader.rs)

- Tokens: `{` `}` `=` `.` , numeric literals, words.
- Words: `[a-zA-Z_][a-zA-Z0-9_]*`.
- Numbers: decimal digits; a literal whose value exceeds 255 is a scan error
  ("number literal exceeds 255") — the token type is `u8`.
- Comments: `#` to end of line. Whitespace separates tokens.
- Any other byte is `MalformedSource` ("illegal character in source").
- No regex; a hand-rolled byte scanner tracks byte offsets, and `line_col`
  converts offsets to 1-based line/column for every rejection.

### Reserved words

Exactly eight, from `is_reserved` (loader.rs):

```
activity  sequence  xor  parallel  loop  redo  max  silent
```

Reserved words are rejected as activity names, source names, and event names
("reserved word used as activity/source/event name"). They may not appear as
activity references either (they parse as keywords first).

### Grammar (EBNF, transcribed from parse_document / parse_expr)

```ebnf
document   = { activity_decl } root_block ;
activity_decl = "activity" NAME "=" NAME "." NAME ;
root_block = expr ;
expr       = sequence | xor | parallel | loop | "silent" | NAME_REF ;
sequence   = "sequence" "{" { expr } "}" ;
xor        = "xor" "{" { expr } "}" ;
parallel   = "parallel" "{" { expr } "}" ;
loop       = "loop" "{" expr "}" [ "redo" "{" expr "}" ] [ "max" NUMBER ] ;
NAME       = word , not reserved , not already declared (as activity name) ;
NAME_REF   = word , must be a declared activity name ;
NUMBER     = digit { digit } , value in 0..=255 ;
```

Semantics pinned by the parser:

- The document is zero or more `activity` declarations followed by exactly
  one root expression. A document with declarations but no root refuses
  ("missing root expression"). Tokens after the root refuse ("unexpected
  trailing tokens after root expression"; a stray `}` refuses as "unbalanced
  '}' with no matching '{'").
- `redo` may be omitted; the redo branch is then `PowlNode::Silent`.
- `max` may be omitted; it defaults to 255.
- Blocks nest arbitrarily (`parse_children` recurses through `parse_expr`).
- An undeclared name in expression position refuses with `UnknownActivity`
  (line/column of the reference).

### Node cap

`MAX_POWL_NODES: usize = 64` (loader.rs:47). Counted after parse via
`count_nodes` over the root (root included; `Loop` counts 1 + body + redo).
Exceeding it refuses `ConformanceError::ModelTooLarge { found, limit }`.
The merger fixture nets report their budget in a header comment
(phase1_intake: 34/64; master: 11/64).

### Model types

```rust
pub struct PowlModel {
    pub activities: BTreeMap<String, ActivityDecl>, // name -> {source, event}
    pub declaration_order: Vec<String>,
    pub root: PowlNode,
}

pub enum PowlNode {
    Sequence(Vec<Self>),
    Xor(Vec<Self>),
    Parallel(Vec<Self>),
    Loop { body: Box<Self>, redo: Box<Self>, max: u8 },
    Activity(String),
    Silent,
}
```

`PowlNode`'s serde impl is hand-written (not derived): every variant serializes
with a `kind` tag in snake_case, e.g. `{"kind":"sequence","children":[...]}`
and `{"kind":"activity","name":"x"}`, and round-trips exactly.

### Public API (loader.rs)

```rust
impl PowlModel {
    pub fn node_count(&self) -> usize;
    pub fn xor_arity_profile(&self) -> Vec<usize>; // DFS, left-to-right

    pub fn from_powl_str(src: &str)
        -> Result<Self, ConformanceError>;
    pub fn from_powl_file(path: impl AsRef<Path>)
        -> Result<Self, ConformanceError>;   // ConformanceError::Io on unreadable
    pub fn from_powl_json(json: &str)
        -> Result<Self, ConformanceError>;   // InvalidJson on bad JSON
}
```

### Position-typed errors (src/conformance/error.rs)

Every parse rejection names its 1-based line and column:

```rust
MalformedSource { line: u32, col: u32, reason: &'static str }
UnknownActivity { name: String, line: u32, col: u32 }
DuplicateActivity { name: String, line: u32, col: u32 }
```

Loader-adjacent errors on the same enum: `InvalidJson { message }`,
`Io { path, message }`; validation-side errors include `ModelTooLarge`,
`UnboundedLoop`, `ArityBelowMinimum`, `DependencyCycle`, `UnknownNodeRef`,
`WeightedArc`, `TooManyPlaces`. The enum is feature-agnostic: foreign error
types are captured as plain messages, never `#[from]`-bound.

### Worked example (landed fixture)

`examples/merger_matrix/models/phase1_intake.powl`, excerpt:

```text
activity a01_shacl_ingest       = fibo_fnd.entity_graph
activity a03_covenant_conflicts = lkif.open_texture_conflict

sequence {
  a01_shacl_ingest
  parallel {
    sequence { a02_subsidiary_closure
               xor { a03_covenant_conflicts silent } }
    sequence { a04_naics_map
               loop { a05_cet1_buffers } redo { silent } max 3 }
    sequence { a06_isda_positions
               a07_facilities
               a08_repo_collateral
               loop { a09_intercompany } redo { silent } max 255
               a10_irc368_election }
  }
  sequence { a11_supervisor_init a12_temprun_leaf a13_pn64_load }
  sequence { a14_merkle_root a15_lease_issue }
  parallel { court_n01 court_n05 court_n09 court_n13 court_n17 }
}
```

## Part 2: AshGraphLaw capability surface

All capability modules are GENERATED by ggen from the GraphLaw capability
registry (header: "DO NOT EDIT"). They share one shape; per-operation
field/variant/refusal metadata lives in `AshGraphLaw.Capability.Registry`.

### Common function signatures (identical across modules)

```elixir
@spec op() :: String.t()                                    # wire name, e.g. "shacl"
@spec fields(:request | :response) :: [Registry.field()]
@spec build_request(AshGraphLaw.Capability.args()) ::
  {:ok, map()} | {:error, Refusal.t()}
@spec decode(term()) :: {:ok, t()} | {:error, Refusal.t()}
@spec run(AshGraphLaw.Capability.args(), keyword()) ::
  {:ok, t()} | {:error, Refusal.t()}
```

`run/2` = `build_request` -> `AshGraphLaw.call/2` (same options: server,
timeouts, fuel) -> `decode`, inside a `[:ash_graphlaw, :capability, ...]`
telemetry span. `build_request` refuses `:invalid_capability_request` on
unknown keys, missing required fields, or type mismatches; `decode` refuses
`:capability_response_undecodable` on a non-map response.

### `AshGraphLaw.Capability.Shacl` (wire op `"shacl"`, registry order 7)

Source: /Users/sac/ash_graphlaw/lib/ash_graphlaw/capability/shacl.ex.
Request fields:

| field | type | required | notes |
|---|---|---|---|
| `data` | data_spec | yes | object `{text, dialect?, hint?, base?}`; dialect sniffed when omitted |
| `shapes` | string | yes | SHACL shapes graph as Turtle text |
| `base` | string | no | base IRI |

Response: `conforms` (boolean), `results`
(list of `{focus, path, value, severity, component, shape, message}`).
Result type `AshGraphLaw.Result.Shacl`.

### `AshGraphLaw.Capability.N3` (wire op `"n3"`)

Same module shape; request/response fields declared in its
`@request_fields`/`@response_fields` and `Registry.op/1`. Consult the module
for exact fields — this doc cites the file rather than restating a field list
that only the registry owns.

### Other modules in `lib/ash_graphlaw/capability/` (including `capabilities.ex`, registry order 1)

`Api, CanonicalJson, Canonical, Coerce, Convert, Datalog, Decode, Entail,
Hooks, Law, Limits, N3, Parse, Policy, Registry, Shacl, Shex, Sniff, Sparql`.
Each capability module follows the identical five-function shape above.
`Coerce.request/3` owns argument coercion; `Decode.ensure_map/1` owns
response shaping.

### Divergence notes

- `shacl.ex:142` is `run/2` as cited elsewhere in this doc set; the line
  number is as-of today's checkout and may move — the `@spec` is the stable
  contract.
- Field lists here are restated for convenience; the generated modules and
  `AshGraphLaw.Capability.Registry` are canonical. If this doc and the
  registry disagree, the registry wins.
