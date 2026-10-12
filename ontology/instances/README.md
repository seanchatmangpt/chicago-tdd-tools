# Instances — KNOWN-BROKEN (W27 probe)

These instance files are NOT loadable:

- `academic-lifecycle.ttl` / `claims-processing.ttl` are RDF/XML misnamed
  `.ttl`, AND malformed (`</ac:datatype>` closes `<ac:lastUpdated>` at :31).
- `chatman-paper.rdf` is not well-formed XML (unescaped `&&` at :303).
- The `KnowledgeHooks` operator names reference workflow-stage names, none of
  which exist in `src/operator_registry.rs`'s 43 YAWL patterns.

Preserved as provenance for the regenerate-paper templates; do not parse.
