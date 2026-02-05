# `wasmtestreport.py` compile-flag behavior

- Uses `--compile-flags` to pass extra flags to both lind and native builds.
- Parses compile flags that begin with `-` correctly (values are collected until the next `--` option).
- Supports a `--dir-flags` JSON file that maps directory prefixes to extra lind/native flags.
- Automatically appends `-lm` for tests under the configured math directory (`MATH_TEST_DIR`, defaults to `math`).
- Per-directory `compile_flags.json` files are not used or copied into artifacts.
