#!/bin/bash

# Requires lind-boot to be built with the `lind_perf` feature. 
# Use `make lind-boot-perf` for this.

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

"${SCRIPT_DIR}/lind_compile" "${BENCH_ROOT}/libc_syscall.c" &>/dev/null && mv "${BENCH_ROOT}/libc_syscall.wasm" "${REPO_ROOT}/lindfs/"
"${SCRIPT_DIR}/lind_compile" "${BENCH_ROOT}/fdtables_syscall.c" &>/dev/null && mv "${BENCH_ROOT}/fdtables_syscall.wasm" "${REPO_ROOT}/lindfs/"
"${SCRIPT_DIR}/lind_compile" --compile-grate "${BENCH_ROOT}/grate_syscall.c" &>/dev/null && mv "${BENCH_ROOT}/grate_syscall.wasm" "${REPO_ROOT}/lindfs/"

echo -en "\nLIBC Test\t"
sudo lind-boot --perf libc_syscall.wasm

echo -en "\nFDTABLE Test\t"
sudo lind-boot --perf fdtables_syscall.wasm

echo -en "\nGRATE Test\t"
sudo lind-boot --perf grate_syscall.wasm libc_syscall.wasm
