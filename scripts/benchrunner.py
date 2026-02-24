#!/usr/bin/env python3
import json
import os
import sys
import subprocess
from pathlib import Path


def print_usage():
    """Print CLI usage."""
    msg = (
        "Usage: benchrunner.py [--out FILE] [TEST_PREFIX]\n\n"
        "Options:\n"
        "  -o, --out FILE   Write results as JSON to FILE\n"
        "  -h, --help       Show this help message\n\n"
        "Args:\n"
        "  TEST_PREFIX      Run only tests that match PREFIX*.c)\n"
    )
    print(msg)


def compile_lind(path):
    """Compile a C benchmark to wasm using lind_compile."""
    status = subprocess.run(["lind_compile", path, "tests/benchmarks/bench.c"],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    if status.returncode:
        print(status.stderr.decode('utf-8'))
        print(status.stdout.decode('utf-8'))
        os._exit(1)

    return f'{path.replace("tests/benchmarks", "")}wasm'


def compile_native(path):
    """Compile a C benchmark to a native binary using cc."""
    status = subprocess.run(["cc", path, "tests/benchmarks/bench.c", "-o",
                             f"{path.replace('.c', '')}"],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    if status.returncode:
        print(status.stderr.decode('utf-8'))
        print(status.stdout.decode('utf-8'))
        os._exit(1)

    return f'{path.replace(".c", "")}'


def run_test(res, path, lind=False):
    """Run a compiled benchmark and append results to the res dict."""
    run_cmd = []

    if lind:
        run_cmd.append("sudo")
        run_cmd.append("lind-boot")

    run_cmd.append(path)

    status = subprocess.run(run_cmd, timeout=180,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    if status.returncode:
        print(status.stderr.decode('utf-8'))
        os._exit(1)

    for line in status.stdout.decode('utf-8').splitlines():
        if len(line.split("\t")) != 4:
            continue
        test, param, loops, avg = [part.strip() for part in line.split("\t")]

        platform = 'lind' if lind else 'linux'

        if test not in res.keys():
            res[test] = {}

        if param not in res[test].keys():
            res[test][param] = {'linux': 0, 'lind': 0, 'loops': 0}

        res[test][param][platform] = avg
        res[test][param]['loops'] = loops


def print_results(res):
    """Print results as a padded table."""
    rows = []
    for test in res.keys():
        for param in res[test].keys():
            linux = res[test][param]['linux']
            lind = res[test][param]['lind']
            loops = res[test][param]['loops']
            rows.append((test, param, linux, lind, loops))

    rows.sort(key=lambda r: (r[1], r[0]))

    headers = ("TEST", "PARAM", "LINUX (ns)", "LIND (ns)", "ITERATIONS")
    widths = [len(h) for h in headers]
    for row in rows:
        for i, val in enumerate(row):
            widths[i] = max(widths[i], len(str(val)))

    fmt = "  ".join([f"{{:<{w}}}" for w in widths])
    print(fmt.format(*headers))
    print("  ".join(["-" * w for w in widths]))
    for row in rows:
        print(fmt.format(*row))


def write_json(res, path):
    """Write results as JSON to a file."""
    with open(path, "w", encoding="utf-8") as f:
        json.dump(res, f, indent=2, sort_keys=True)


if __name__ == "__main__":
    test_pattern = "*.c"
    output_json = None
    if len(sys.argv) > 1:
        if sys.argv[1] in ("-h", "--help"):
            print_usage()
            sys.exit(0)
        if sys.argv[1] in ("-o", "--out"):
            if len(sys.argv) < 3:
                print("error: --out requires a file path", file=sys.stderr)
                sys.exit(2)
            output_json = sys.argv[2]
            if len(sys.argv) > 3:
                test_pattern = f"{sys.argv[3]}*.c"
        else:
            test_pattern = f"{sys.argv[1]}*.c"

    bench_dir = Path("tests/benchmarks/")
    bench_tests = [x for x in list(
        map(str, bench_dir.glob(test_pattern))) if not x.endswith("bench.c")]

    res = dict()

    for test in bench_tests:
        native_path = compile_native(test)
        lind_path = compile_lind(test)

        run_test(res, lind_path, True)
        run_test(res, native_path)

    if output_json:
        write_json(res, output_json)
    else:
        print_results(res)
