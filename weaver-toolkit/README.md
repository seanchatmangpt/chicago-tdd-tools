# weaver-toolkit

Reusable, vendorable convenience wrappers around the
[`weaver`](https://github.com/open-telemetry/weaver) CLI's `registry
live-check`, factored out of chicago-tdd-tools so other projects can consume
them without depending on this repo's Rust workspace.

This directory has no dependency on the rest of chicago-tdd-tools — copy it,
`git subtree`/submodule it, or reference it in place.

## Contents

| File | Purpose |
|------|---------|
| `weaver-wrapper.sh` | Standalone bash script: bootstrap, static `check`, live-check `live-start`/`live-stop`, `version`. |
| `weaver.just` | `just` module exposing the same operations as recipes, for projects that already use `just`. |
| `docker-compose.weaver.yml` | Reusable compose service for a standing live-check container (builds from `../Dockerfile.weaver` by default). |

## Option A — plain shell, no `just`

```bash
cp -r weaver-toolkit /path/to/other-project/weaver-toolkit
cd /path/to/other-project
./weaver-toolkit/weaver-wrapper.sh bootstrap
./weaver-toolkit/weaver-wrapper.sh check
./weaver-toolkit/weaver-wrapper.sh live-start
# ...emit OTLP spans at 127.0.0.1:4319...
./weaver-toolkit/weaver-wrapper.sh live-stop   # writes ./weaver-reports/report.json
```

Everything is configurable via env vars documented at the top of
`weaver-wrapper.sh` (`WEAVER_VERSION`, `WEAVER_HOME`,
`WEAVER_REGISTRY_PATH`, `WEAVER_OTLP_GRPC_PORT`, `WEAVER_ADMIN_PORT`,
`WEAVER_REPORTS_DIR`). Set `WEAVER_REGISTRY_PATH` to your own local registry
checkout to skip the default OTel semantic-conventions clone.

## Option B — `just` projects

In the consuming project's `Justfile`:

```just
import 'weaver-toolkit/weaver.just'
```

Then:

```bash
just weaver-toolkit-bootstrap
just weaver-toolkit-check
just weaver-toolkit-live-start
just weaver-toolkit-live-stop
```

## Option C — standing container (Docker/Kubernetes-adjacent local dev)

```bash
docker compose -f weaver-toolkit/docker-compose.weaver.yml up -d
```

Point `WEAVER_BUILD_CONTEXT` at wherever your `Dockerfile.weaver` and
`registry/model` live if you're not building from this repo's root, or set
`WEAVER_IMAGE` to skip the build and pull a pre-built image instead. See
`../Dockerfile.weaver` for the image this builds by default — it bundles the
official weaver musl binary with a semantic-convention registry and starts
`registry live-check` with a long inactivity timeout so it stays up as a
standing service.

## Verification (from outside Rust)

`verify-python/` is a self-contained [uv](https://docs.astral.sh/uv/)
project that Chicago-style black-box tests `weaver-wrapper.sh` and the OTLP
listener it starts, driven purely as a shell CLI + real OTLP client — no
Rust, no mocks. See `verify-python/README.md`. Run it with
`just weaver-toolkit-verify-python`.

## Why a wrapper instead of a Cargo dependency

`weaver` is an external Go/Rust upstream binary, not code this repo owns —
wrapping it in shell/`just`/compose keeps it usable from non-Rust projects,
and keeps the wrapper's only real dependency (`curl`/`wget`, `tar`, `git`,
`timeout`) something every dev environment already has.
