## What changed
This PR adds lightweight per-test runtime reporting to `scripts/wasmtestreport.py` so test duration is visible in the generated report.

### 1) Capture elapsed time per test case
- In `run_tests`, each deterministic and fail test execution is wrapped with `time.perf_counter()`.
- The elapsed wall-clock runtime is computed and rounded to milliseconds precision (`round(..., 3)`).
- The elapsed value is stored on that test case entry as `elapsed_seconds`.

### 2) Show timing in HTML output
- The HTML test results table now includes a `Duration (s)` column.
- Category header `colspan` was updated from 4 to 5 to match the extra column.
- Each row renders `elapsed_seconds` when available, and falls back to `N/A` when timing is absent.

## Why
Issue #668 asked for runtime visibility per test to help compare behavior/performance and tune timeouts. This provides that signal with minimal, low-risk changes.

## Scope
- Single file touched: `scripts/wasmtestreport.py`
- Additive behavior only: no test selection/execution logic changed beyond recording and displaying elapsed runtime metadata.

## Validation
- `python3 -m py_compile scripts/wasmtestreport.py`
