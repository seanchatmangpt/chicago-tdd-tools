#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
exec "$ROOT/scripts/run_real_ggen_cell.sh" architecture/combinatorial-maximalism
