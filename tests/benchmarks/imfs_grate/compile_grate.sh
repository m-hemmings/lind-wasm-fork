#!/usr/bin/env bash

# Usage: ./compile_grate.sh <example-dir>
#
# Builds and outputs a WebAssembly binary for lind.
#
# Expected directory structure:
# <example-dir>/
#     ├── build.conf       (Required: ENTRY, Optional: MAX_MEMORY, EXTRA_CFLAGS)
#     └── src/
#         └── *.c          (Source files to compile)
#
# Outputs to <example-dir>/output/:
#   - <entry>.wasm
#   - <entry>.cwasm

set -euo pipefail

cd "$(dirname "$0")"
lind_compile --compile-grate src/imfs_grate.c src/imfs.c
