# External Contracts

> Manufactured from `ontology.ttl`. Do not edit.

| Order | ID | Protocol | Source realization | Authority | Receipt | Replay | Private leak |
|---:|---|---|---|---|:---:|:---:|:---:|
| 1 | `contract.otlp` | `otlp` | `obs.otel` | `Observer` | true | true | false |
| 2 | `contract.ocel` | `ocel-2.0` | `obs.ocel` | `Observer` | true | true | false |
| 3 | `contract.cli` | `stdio` | `verify.cli` | `Observer` | true | true | false |
| 4 | `contract.docker` | `docker-api` | `verify.integration` | `Observer` | true | true | false |
| 5 | `contract.container-broker` | `docker-api` | `actuate.testcontainers` | `Brokered` | true | true | false |
| 6 | `contract.sector-broker` | `rust-api` | `actuate.sector` | `Brokered` | true | true | false |
| 7 | `contract.git-hook` | `git-hook-intent` | `actuate.git_hook` | `Brokered` | true | true | false |
| 8 | `contract.ocel-receipt` | `ocel-2.0` | `receipt.ocel` | `Observer` | true | true | false |
| 9 | `contract.ocel-replay` | `ocel-2.0` | `replay.ocel` | `Observer` | true | true | false |
| 10 | `contract.verifier-report` | `verifier-report-v1` | `replay.verifier_report` | `Observer` | true | true | false |

