# weaver.just — reusable weaver recipes for consuming projects.
#
# Vendor this file (git submodule, subtree, or plain copy of weaver-toolkit/)
# and pull it into your own Justfile with:
#
#   import 'path/to/weaver-toolkit/weaver.just'
#
# All recipes delegate to weaver-wrapper.sh, which is self-contained and has
# no dependency on this repo's Cargo workspace.

# source_directory() resolves relative to this file's own location, not the
# importing project's justfile root, so this module works no matter where a
# consuming project vendors weaver-toolkit/.
weaver_toolkit_wrapper := source_directory() / "weaver-wrapper.sh"

# Download the weaver binary + semantic-conventions registry into ./.weaver
weaver-toolkit-bootstrap:
    bash {{weaver_toolkit_wrapper}} bootstrap

# Static validation of the registry (no OTLP listener)
weaver-toolkit-check: weaver-toolkit-bootstrap
    bash {{weaver_toolkit_wrapper}} check

# Start an ephemeral OTLP live-check listener in the background
weaver-toolkit-live-start: weaver-toolkit-bootstrap
    bash {{weaver_toolkit_wrapper}} live-start

# Stop the live-check listener and collect its JSON report
weaver-toolkit-live-stop:
    bash {{weaver_toolkit_wrapper}} live-stop

# Print the installed weaver binary's version
weaver-toolkit-version:
    bash {{weaver_toolkit_wrapper}} version

# Black-box test this toolkit from outside Rust (uv/pytest) -- see verify-python/README.md
weaver-toolkit-verify-python:
    cd {{source_directory()}}/verify-python && uv sync && uv run pytest -v
