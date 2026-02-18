## ISSUE
- The test/reporting flow had diverged across scripts and CI wiring, making it hard to run multiple harnesses consistently and expose one unified artifact for GitHub Actions/PR visibility.
- Grate tests needed to follow the same harness contract and operational behavior as wasm tests (including timeout handling, consistent success/failure logging, and wrapper-based runtime invocation).
- CI and local `make test` were still oriented around direct `wasmtestreport.py` usage instead of a unified runner with combined output.

## CHANGES
- Added a unified harness runner at `scripts/test_runner.py` that:
  - discovers harness modules under `scripts/harnesses/`,
  - runs selected harnesses through a common `run_harness(...)` interface,
  - writes per-harness artifacts to `reports/`, and
  - generates a combined `reports/report.html` aggregating all harness outputs.
- Added `--export-report` support to `scripts/test_runner.py` so CI can copy the combined report to `/report.html` for downstream PR/comment workflows.
- Added/updated harness modules in `scripts/harnesses/`:
  - `wasmtestreport.py` (standalone harness copy aligned to updated behavior),
  - `gratetestreport.py` (grate harness implementation with timeout support, strict success-on-exit-code-0 semantics, and detailed failure capture),
  - `__init__.py` for package/module discovery.
- Updated grate harness behavior to align operationally with wasm flow:
  - prefers `scripts/lind_run` (with environment override support),
  - resolves generated `.wasm` outputs robustly,
  - emits explicit per-test `SUCCESS` / `FAILURE: ...` style logging,
  - uses `grates.json` as the report artifact name.
- Updated repo integration points to use the unified runner and expose new artifacts:
  - `Makefile` test target now uses `scripts/test_runner.py` and exports combined HTML,
  - `Docker/Dockerfile.e2e` updated for unified-runner script usage/artifacts,
  - `.github/workflows/e2e.yml` updated to upload/report unified artifacts (including combined HTML and harness JSON outputs).

## NOTES
- Timeout/slowness symptoms were observed before the unified runner migration and appear to be broader runtime behavior rather than caused solely by the harness orchestration layer.
- The combined report path is now `reports/report.html` (with optional export to `/report.html`), while per-harness files remain separately available for debugging.
- Grate harness success criteria is intentionally strict: only process exit code `0` is success; all non-zero exits are reported as failures with captured output.
