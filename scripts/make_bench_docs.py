#!/usr/bin/env python3
"""Run the `public` benchmark suite and write the results into the docs folder."""

import os
import subprocess
from pathlib import Path

import cpuinfo

BENCH_NAME = "public"

DEFAULT_BENCH_FILEPATH = Path("./docs/bench-default.md")
NATIVE_BENCH_FILEPATH = Path("./docs/bench-native.md")


def get_cpu_name() -> str:
    """Return the marketing name of the host CPU."""
    return cpuinfo.get_cpu_info()["brand_raw"]


def run_bench(name: str, target_cpu: str | None) -> str:
    """Run the named cargo benchmark and return its captured result table.

    When ``target_cpu`` is given it is forwarded to rustc via ``-C target-cpu``.
    """
    env = os.environ.copy()
    if target_cpu:
        env["RUSTFLAGS"] = f"-C opt-level=3 -C target-cpu={target_cpu}"
    else:
        env["RUSTFLAGS"] = "-C opt-level=3"

    output: list[str] = []
    do_capture = False

    with subprocess.Popen(
        ["cargo", "bench", name],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        env=env,
        encoding="utf-8",
    ) as p:
        if p.stdout is None:
            msg = "failed to capture cargo bench output"
            raise RuntimeError(msg)

        for line in p.stdout:
            print(line, end="", flush=True)

            # Omit compilation and info. The first captured line looks like:
            # public  fastest │ slowest │ median │ mean │ samples │ iters
            if line.startswith(name):
                do_capture = True

            if do_capture:
                output.append(line)

        ret = p.wait()

    if ret != 0:
        msg = "cargo bench failed"
        raise RuntimeError(msg)

    return "".join(output)


def main() -> None:
    """Generate the default- and native-target benchmark documents."""
    bench_default = run_bench(BENCH_NAME, None)
    bench_native = run_bench(BENCH_NAME, "native")

    with DEFAULT_BENCH_FILEPATH.open("w", encoding="utf-8") as outfile:
        outfile.write(f"# `{BENCH_NAME}` bench with default target\n")
        outfile.write(f"```\n{bench_default}\n```\n")

    with NATIVE_BENCH_FILEPATH.open("w", encoding="utf-8") as outfile:
        cpu = get_cpu_name()
        outfile.write(f"# `{BENCH_NAME}` bench with native target ({cpu})\n")
        outfile.write(f"```\n{bench_native}\n```\n")


if __name__ == "__main__":
    main()
