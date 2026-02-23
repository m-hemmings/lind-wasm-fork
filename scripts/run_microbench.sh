#!/bin/bash

set -euo pipefail

# Check if we need to re-exec with sudo
if [[ $EUID -ne 0 ]]; then
  # Not running as root, re-exec with sudo
  exec sudo -E "$0" "$@"
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="${SCRIPT_DIR%scripts}"
BENCH_ROOT="${REPO_ROOT}/tests/benchmarks"

echo "Compiling Tests..."

"${SCRIPT_DIR}/lind_compile" "${BENCH_ROOT}/libccall.c" &>/dev/null && mv "${BENCH_ROOT}/libccall.wasm" "${REPO_ROOT}/lindfs/"
"${SCRIPT_DIR}/lind_compile" "${BENCH_ROOT}/fdtcall.c" &>/dev/null && mv "${BENCH_ROOT}/fdtcall.wasm" "${REPO_ROOT}/lindfs/"
"${SCRIPT_DIR}/lind_compile" --compile-grate "${BENCH_ROOT}/gratecall.c" &>/dev/null && mv "${BENCH_ROOT}/gratecall.wasm" "${REPO_ROOT}/lindfs/"

echo -en "\nLIBC Test\t"
sudo lind-boot --perf libccall.wasm

echo -en "\nFDTABLE Test\t"
sudo lind-boot --perf fdtcall.wasm

echo -en "\nGRATE Test\t"
sudo lind-boot --perf gratecall.wasm libccall.wasm
