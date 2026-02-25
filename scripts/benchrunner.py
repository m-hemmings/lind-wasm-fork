#!/usr/bin/env python3
import argparse
import json
import os
import subprocess
from pathlib import Path


def repo_root() -> Path:
    """Return repo root (scripts/..)."""
    return Path(__file__).resolve().parent.parent


ROOT = repo_root()
BENCH_DIR = ROOT / "tests" / "benchmarks"
LIND_FS = ROOT / "lindfs"


def run_cmd(cmd, timeout=180):
    """Run a command and return CompletedProcess, exiting on failure."""
    try:
        status = subprocess.run(cmd, timeout=timeout,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except Exception:
        return None

    if status.returncode:
        sys_stderr = status.stderr.decode("utf-8")
        sys_stdout = status.stdout.decode("utf-8")
        if sys_stderr:
            print(sys_stderr)
        if sys_stdout:
            print(sys_stdout)
        os._exit(1)
    return status


def bench_relpath(path: Path) -> Path:
    """Return path relative to tests/benchmarks."""
    return path.resolve().relative_to(BENCH_DIR)


def lindfs_path(rel: Path) -> Path:
    """Return absolute path inside lindfs for a relative benchmark path."""
    return LIND_FS / rel


def compile_lind(c_file: Path) -> str:
    """Compile a C benchmark to wasm using lind_compile."""
    run_cmd(["lind_compile", str(c_file), str(
        BENCH_DIR / "imfs.c"), str(BENCH_DIR / "bench.c")])

    rel = bench_relpath(c_file).with_suffix(".cwasm")
    # lind_compile places outputs inside lindfs; lind-boot is chrooted there.
    return rel.as_posix()


def compile_native(c_file: Path) -> Path:
    """Compile a C benchmark to a native binary and place it in lindfs."""
    rel = bench_relpath(c_file).with_suffix("")
    out_path = lindfs_path(rel)
    out_path.parent.mkdir(parents=True, exist_ok=True)

    run_cmd(
        [
            "cc",
            str(c_file),
            str(BENCH_DIR / "imfs.c"),
            str(BENCH_DIR / "bench.c"),
            "-o",
            str(out_path),
        ]
    )
    return out_path


def compile_grate(grate_dir: Path) -> str:
    """Compile a grate folder and return the output path inside lindfs."""
    run_cmd(["bash", str(grate_dir / "compile_grate.sh"), "."])
    rel = bench_relpath(grate_dir).with_suffix(".cwasm")
    return rel.as_posix()


def parse_output(res, output, platform):
    """Parse benchmark output lines and update results."""
    try:
        for line in output.decode("utf-8").splitlines():
            parts = [part.strip() for part in line.split("\t")]
            if len(parts) != 4:
                continue
            test, param, loops, avg = parts

            param = int(param)

            if test not in res:
                res[test] = {}
            if param not in res[test]:
                res[test][param] = {"linux": -1,
                                    "lind": -1, "grate": -1, "loops": -1}

            res[test][param][platform] = avg
            res[test][param]["loops"] = loops
    except:
        print("Invalid output from test")


def run_lind(wasm_paths, res, platform):
    """Run lind-boot with one or more wasm paths."""
    cmd = ["sudo", "lind-boot"] + wasm_paths
    status = run_cmd(cmd)
    if not status:
        return

    parse_output(res, status.stdout, platform)


def run_native(binary_path: Path, res):
    """Run a native benchmark binary."""
    status = run_cmd([str(binary_path)])
    if not status:
        return
    parse_output(res, status.stdout, "linux")


def run_grate_test(grate_dir: Path, res):
    """Run a grate test described by a .grate file or directory."""
    bins = []

    for part in grate_dir.name.split("."):
        if part.endswith("_grate"):
            bins.append(compile_grate(Path(BENCH_DIR) / part))
        else:
            c_file = BENCH_DIR / f"{part}.c"
            bins.append(compile_lind(c_file))

    run_lind(bins, res, "grate")


def to_int(value):
    """Best-effort int conversion for numeric strings."""
    if isinstance(value, int):
        return value
    try:
        return int(value)
    except (TypeError, ValueError):
        return -1


def format_ratio(value, base):
    """Format value and its ratio to base."""
    v = to_int(value)
    b = to_int(base)
    if v < 0:
        return "--"
    if b <= 0:
        return str(value)
    return f"{v} ({v / b:.3f})"


def print_results(res):
    """Print results as a padded table sorted by test and param."""
    rows = []
    for test in res:
        for param in res[test]:
            linux = res[test][param]["linux"]
            lind = res[test][param]["lind"]
            grate = res[test][param]["grate"]
            loops = res[test][param]["loops"]

            rows.append(
                (
                    test,
                    param,
                    linux if linux != -1 else "--",
                    format_ratio(lind, linux),
                    format_ratio(grate, linux),
                    loops,
                )
            )

    rows.sort(key=lambda r: (r[0], r[1]))

    headers = ("TEST", "PARAM", "LINUX (ns)",
               "LIND (ns)", "GRATE (ns)", "ITERATIONS")
    widths = [len(h) for h in headers]
    for row in rows:
        for i, val in enumerate(row):
            widths[i] = max(widths[i], len(str(val)))

    fmt = "  ".join([f"{{:<{w}}}" for w in widths])
    print(fmt.format(*headers))
    print("  ".join(["-" * w for w in widths]))
    for row in rows:
        print(fmt.format(*row))


def write_json(res, path: Path):
    """Write results as JSON to a file."""
    with open(path, "w", encoding="utf-8") as f:
        json.dump(res, f, indent=2, sort_keys=True)


def parse_args():
    parser = argparse.ArgumentParser(
        description="Run lind-wasm microbenchmarks")
    parser.add_argument(
        "patterns",
        nargs="*",
        help="Test name prefixes (e.g. fs_ imfs_). Defaults to all.",
    )
    parser.add_argument(
        "-o", "--out", dest="output_json", help="Write results to JSON file"
    )
    return parser.parse_args()


def collect_tests(patterns):
    """Return benchmark paths matching patterns."""
    if not patterns:
        patterns = [""]
    files = []
    for p in patterns:
        for path in BENCH_DIR.glob(f"{p}*"):
            if path.name in ("bench.c", "imfs.c"):
                continue
            if path.is_file() or path.suffix == ".grate":
                files.append(path)
    return files


def main():
    args = parse_args()
    tests = collect_tests(args.patterns)
    res = {}

    for test in tests:
        if test.suffix == ".c":
            print("Running: ", test)
            native_path = compile_native(test)
            lind_path = compile_lind(test)
            run_lind([lind_path], res, "lind")
            run_native(native_path, res)
        elif test.suffix == ".grate":
            print("Running: ", test)
            run_grate_test(test.with_suffix(""), res)

    if args.output_json:
        write_json(res, Path(args.output_json))
    else:
        print_results(res)


if __name__ == "__main__":
    main()
