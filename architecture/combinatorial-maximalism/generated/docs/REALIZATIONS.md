# Realization Lattice

> Manufactured from `ontology.ttl`. Do not edit.

| Order | ID | Facet | Module | Visibility | Authority | Effect | Protocol | Dependency |
|---:|---|---|---|---|---|---|---|---|
| 1 | `obs.fixture` | `facet.observation` | `core::fixture` | `Internal` | `Pure` | `none` | `rust` | `NONE` |
| 2 | `obs.contract` | `facet.observation` | `core::contract` | `Internal` | `Pure` | `none` | `rust` | `NONE` |
| 3 | `obs.otel` | `facet.observation` | `observability::otel` | `External` | `Observer` | `network` | `otlp` | `NONE` |
| 4 | `obs.ocel` | `facet.observation` | `observability::ocel` | `External` | `Observer` | `storage` | `ocel-2.0` | `NONE` |
| 5 | `admit.type_level` | `facet.admission` | `core::type_level` | `Internal` | `Pure` | `none` | `rust` | `obs.contract` |
| 6 | `admit.invariants` | `facet.admission` | `core::invariants` | `Internal` | `Pure` | `none` | `rust` | `obs.fixture` |
| 7 | `admit.governance` | `facet.admission` | `core::governance` | `Internal` | `Pure` | `none` | `rust` | `obs.contract` |
| 8 | `admit.guards` | `facet.admission` | `validation::guards` | `Internal` | `Pure` | `none` | `rust` | `obs.contract` |
| 9 | `construct.builders` | `facet.construction` | `core::builders` | `Internal` | `Pure` | `none` | `rust` | `admit.type_level` |
| 10 | `construct.generator` | `facet.construction` | `testing::generator` | `Internal` | `Pure` | `none` | `rust` | `admit.type_level` |
| 11 | `construct.operator_registry` | `facet.construction` | `operator_registry` | `Internal` | `Pure` | `none` | `rust` | `admit.invariants` |
| 12 | `construct.swarm` | `facet.construction` | `swarm` | `Internal` | `Pure` | `none` | `rust` | `admit.governance` |
| 13 | `verify.assertions` | `facet.verification` | `core::assertions` | `Internal` | `Pure` | `none` | `rust` | `admit.invariants` |
| 14 | `verify.property` | `facet.verification` | `testing::property` | `Internal` | `Pure` | `none` | `rust` | `verify.assertions` |
| 15 | `verify.snapshot` | `facet.verification` | `testing::snapshot` | `Internal` | `Pure` | `storage` | `rust` | `verify.assertions` |
| 16 | `verify.mutation` | `facet.verification` | `testing::mutation` | `Internal` | `Pure` | `process` | `rust` | `verify.assertions` |
| 17 | `verify.concurrency` | `facet.verification` | `testing::concurrency` | `Internal` | `Pure` | `process` | `rust` | `verify.assertions` |
| 18 | `verify.cli` | `facet.verification` | `testing::cli` | `External` | `Observer` | `process` | `stdio` | `verify.assertions` |
| 19 | `verify.integration` | `facet.verification` | `integration::testcontainers` | `External` | `Observer` | `network+storage+process` | `docker-api` | `verify.assertions` |
| 20 | `verify.performance` | `facet.verification` | `validation::performance` | `Internal` | `Observer` | `none` | `rust` | `verify.assertions` |
| 21 | `authorize.contract` | `facet.authorization` | `core::contract` | `Internal` | `Pure` | `none` | `rust` | `admit.type_level` |
| 22 | `authorize.governance` | `facet.authorization` | `core::governance` | `Internal` | `Pure` | `none` | `rust` | `admit.governance` |
| 23 | `authorize.jtbd` | `facet.authorization` | `validation::jtbd` | `Internal` | `Pure` | `none` | `rust` | `verify.assertions` |
| 24 | `actuate.intent` | `facet.actuation` | `architecture::combinatorial_maximalism` | `Internal` | `Pure` | `none` | `intent` | `authorize.contract` |
| 25 | `actuate.testcontainers` | `facet.actuation` | `integration::testcontainers` | `External` | `Brokered` | `network+storage+process` | `docker-api` | `authorize.governance` |
| 26 | `actuate.sector` | `facet.actuation` | `sector_stacks` | `External` | `Brokered` | `storage` | `rust-api` | `authorize.contract` |
| 27 | `actuate.git_hook` | `facet.actuation` | `git_hook_installer` | `External` | `Brokered` | `filesystem+process` | `git-hook-intent` | `authorize.governance` |
| 28 | `receipt.test` | `facet.receipt` | `core::receipt` | `Internal` | `Pure` | `none` | `receipt-v2` | `verify.assertions` |
| 29 | `receipt.sector` | `facet.receipt` | `sector_stacks::OperationReceipt` | `Internal` | `Pure` | `none` | `receipt-v2` | `actuate.sector` |
| 30 | `receipt.swarm` | `facet.receipt` | `swarm::TaskReceipt` | `Internal` | `Pure` | `none` | `receipt-v2` | `construct.swarm` |
| 31 | `receipt.ocel` | `facet.receipt` | `observability::ocel` | `External` | `Observer` | `storage` | `ocel-2.0` | `obs.ocel` |
| 32 | `replay.pipeline` | `facet.replay` | `core::verification_pipeline` | `Internal` | `Pure` | `none` | `rust` | `receipt.test` |
| 33 | `replay.fail_fast` | `facet.replay` | `core::fail_fast` | `Internal` | `Pure` | `none` | `rust` | `replay.pipeline` |
| 34 | `replay.ocel` | `facet.replay` | `observability::ocel` | `External` | `Observer` | `storage` | `ocel-2.0` | `receipt.ocel` |
| 35 | `replay.verifier_report` | `facet.replay` | `architecture::verifier_report` | `External` | `Observer` | `storage` | `verifier-report-v1` | `replay.pipeline` |

