#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")"
lind_compile --compile-grate geteuid_grate.c
