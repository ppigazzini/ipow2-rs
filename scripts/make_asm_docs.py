#!/usr/bin/env python3
"""Generate per-target assembly and llvm-mca documentation for the ipow2 crate."""

import os
import platform
import re
import subprocess
import sys
from pathlib import Path

ASM_DIR = Path("asm")
ASM_DIR.mkdir(exist_ok=True)


# Things we don't want to dump.
SKIP_PREFIXES: tuple[str, ...] = (
    "core::",
    "std::",
    "alloc::",
    "asm::",
    "<std::",
    "main",
)


WINDOWS_X86_64_TARGET = "x86_64-pc-windows-msvc"
WINDOWS_AARCH64_TARGET = "aarch64-pc-windows-msvc"
LINUX_X86_64_TARGET = "x86_64-unknown-linux-gnu"
LINUX_AARCH64_TARGET = "aarch64-unknown-linux-gnu"

CPUS_X86_64: list[str] = ["znver4", "raptorlake"]
CPUS_AARCH64: list[str] = ["apple-m1"]


def make_mca_filepath(cpu: str) -> Path:
    """Return the output path for the llvm-mca document of ``cpu``."""
    return Path(f"./docs/mca-{cpu}.md")


def make_asm_filepath(target: str, cpu: str) -> Path:
    """Return the output path for the assembly document of ``target``/``cpu``."""
    return Path(f"./docs/asm-{target}-{cpu}.md")


def get_host_os() -> str:
    """Return ``"windows"`` or ``"linux"`` for the host, or raise otherwise."""
    sysname = platform.system().lower()
    if "windows" in sysname:
        return "windows"
    if "linux" in sysname:
        return "linux"
    msg = f"Unsupported OS: {sysname}"
    raise RuntimeError(msg)


def get_target_triples(host_os: str) -> list[str]:
    """Return the native x86_64 and aarch64 Rust target triples for the host OS."""
    if host_os == "windows":
        return [
            WINDOWS_X86_64_TARGET,
            WINDOWS_AARCH64_TARGET,
        ]

    if host_os == "linux":
        return [
            LINUX_X86_64_TARGET,
            LINUX_AARCH64_TARGET,
        ]

    msg = f"No targets defined for OS: {host_os}"
    raise RuntimeError(msg)


def get_installed_targets() -> set[str]:
    """Return the set of Rust targets installed for the nightly toolchain."""
    result = subprocess.run(
        ["rustup", "+nightly", "target", "list", "--installed"],
        capture_output=True,
        text=True,
        check=True,
    )
    return set(result.stdout.splitlines())


def check_targets(
    installed: set[str],
    targets: list[str],
) -> tuple[list[str], list[str]]:
    """Split ``targets`` into (available, missing) against ``installed``."""
    available: list[str] = []
    missing: list[str] = []

    for t in targets:
        if t in installed:
            available.append(t)
        else:
            missing.append(t)

    return available, missing


def strip_trailing_empty_lines(lines: list[str]) -> None:
    """Remove trailing blank lines from ``lines`` in place."""
    while lines and not lines[-1].strip():
        lines.pop()


def extract_llvm_mca_legends(
    stdout: str,
) -> tuple[str, str, str]:
    """Split llvm-mca output into (body, instruction legend, resources legend)."""
    remaining_lines: list[str] = []
    instruction_legend_lines: list[str] = []
    resources_legend_lines: list[str] = []

    lines_output_ptr = remaining_lines

    for line in stdout.splitlines():
        if line.startswith("Instruction Info:"):
            remaining_lines.append(line)
            lines_output_ptr = instruction_legend_lines
        elif line.startswith("Resources:"):
            remaining_lines.append(line)
            lines_output_ptr = resources_legend_lines
        else:
            curr_line_is_empty = line == "" or line.isspace()
            if curr_line_is_empty:
                lines_output_ptr = remaining_lines

            # Do not duplicate empty lines
            if (
                not curr_line_is_empty
                or len(lines_output_ptr) == 0
                or (lines_output_ptr[-1] != "" and not lines_output_ptr[-1].isspace())
            ):
                lines_output_ptr.append(line)

    strip_trailing_empty_lines(remaining_lines)
    strip_trailing_empty_lines(instruction_legend_lines)
    strip_trailing_empty_lines(resources_legend_lines)

    return (
        "\n".join(remaining_lines),
        "\n".join(instruction_legend_lines),
        "\n".join(resources_legend_lines),
    )


def run_llvm_mca(asm: str, arch: str, cpu: str) -> str:
    """Run llvm-mca on ``asm`` for ``arch``/``cpu`` and return its stdout."""
    # NOTE: We can only reliably run a single iteration: we cannot control
    #       register allocation, which can introduce loop-carried dependencies.
    p = subprocess.run(
        [
            "llvm-mca",
            "-x86-asm-syntax=intel",
            f"--march={arch}",
            f"-mcpu={cpu}",
            "-iterations=1",
        ],
        input=asm,
        text=True,
        capture_output=True,
        check=False,
    )

    if p.returncode != 0:
        print("Command failed!")
        print("STDOUT:\n", p.stdout)
        print("STDERR:\n", p.stderr)
        msg = "llvm-mca call failed"
        raise RuntimeError(msg)

    return p.stdout


def make_rustflags_env(cpu: str | None = None) -> dict[str, str]:
    """Return a copy of the environment with RUSTFLAGS set for asm extraction."""
    env = os.environ.copy()

    if cpu:
        env["RUSTFLAGS"] = (
            f"-C opt-level=3 -Z merge-functions=disabled -C target-cpu={cpu}"
        )
    else:
        env["RUSTFLAGS"] = "-C opt-level=3 -Z merge-functions=disabled"

    return env


def cargo_asm_list() -> str:
    """Return the raw ``cargo asm`` listing of all benchmark functions."""
    env = make_rustflags_env()

    cargo_asm_list_cmd = [
        "cargo",
        "+nightly",  # needed for disabling merge-functions
        "asm",
        "--bench",
        "asm",
    ]
    return subprocess.run(
        cargo_asm_list_cmd,
        text=True,
        capture_output=True,
        env=env,
        check=False,
    ).stdout


def cargo_asm(number: str, target: str, cpu: str | None = None) -> str:
    """Return the disassembly of benchmark function ``number`` for a target/cpu."""
    env = make_rustflags_env(cpu)

    cargo_asm_cmd = [
        "cargo",
        "+nightly",  # needed for disabling merge-functions
        "asm",
        "--target",
        target,
        "--simplify",
        "--intel",
        "--bench",
        "asm",
        number,
    ]

    p = subprocess.run(
        cargo_asm_cmd,
        text=True,
        capture_output=True,
        env=env,
        check=False,
    )

    if p.returncode != 0:
        print("Command failed!")
        print("STDOUT:\n", p.stdout)
        print("STDERR:\n", p.stderr)
        msg = "cargo asm call failed"
        raise RuntimeError(msg)

    return p.stdout


def list_functions() -> list[tuple[str, str]]:
    """Return the (number, name) pairs of benchmark functions worth dumping."""
    output = cargo_asm_list()

    functions: list[tuple[str, str]] = []

    for line in output.splitlines():
        # Matches:
        #  2 "div_floor_i128_pow2" [29]
        m = re.match(r'\s*(\d+)\s+"([^"]+)"', line)
        if not m:
            continue

        number = m.group(1)
        name = m.group(2)

        if name.startswith(SKIP_PREFIXES):
            continue

        functions.append((number, name))

    return functions


def strip_preamble(lines: list[str]) -> list[str]:
    """Drop a leading label line (``name:``) from an assembly listing."""
    if lines[0].endswith(":"):
        lines = lines[1:]
    return lines


def strip_trailing_ret(lines: list[str]) -> list[str]:
    """Drop trailing blank lines and a final ``ret`` instruction."""
    while lines and not lines[-1].strip():
        lines.pop()

    if lines and lines[-1].strip().startswith("ret"):
        lines.pop()

    return lines


def sanitize_asm(asm: str) -> str:
    """Strip the label preamble and trailing ``ret`` from an assembly listing."""
    lines = asm.splitlines()

    lines = strip_preamble(lines)
    lines = strip_trailing_ret(lines)

    return "\n".join(lines)


def natural_key(s: str) -> list[int | str]:
    """Return a sort key that orders embedded numbers numerically."""
    return [int(part) if part.isdigit() else part for part in re.split(r"(\d+)", s)]


def sort_functions(functions: list[tuple[str, str]]) -> list[tuple[str, str]]:
    """Return ``functions`` sorted by name using a natural (numeric-aware) order."""
    return sorted(functions, key=lambda x: natural_key(x[1]))


def choose_x86_64_target(targets: list[str]) -> str | None:
    """Return the first x86_64 target in ``targets``, or ``None``."""
    for target in targets:
        if "x86_64" in target:
            return target
    return None


def choose_aarch64_target(targets: list[str]) -> str | None:
    """Return the first aarch64 target in ``targets``, or ``None``."""
    for target in targets:
        if "aarch64" in target:
            return target
    return None


def transpose_asms(asms: dict[str, str]) -> dict[str, list[str]]:
    """Group function names by identical assembly, coalescing duplicates."""
    by_asm: dict[str, list[str]] = {}
    for fn, asm in asms.items():
        if asm in by_asm:
            by_asm[asm].append(fn)
        else:
            by_asm[asm] = [fn]

    for fns in by_asm.values():
        fns.sort(key=natural_key)

    return by_asm


def format_fns(fns: list[str]) -> str:
    """Format a list of function names as a comma-separated code span list."""
    return ", ".join(f"`{fn}`" for fn in fns)


def produce_docs(
    target_x86_64: str,
    target_aarch64: str,
    cpus_x86_64: list[str],
    cpus_aarch64: list[str],
) -> None:
    """Extract assembly and llvm-mca output and write the per-target documents."""
    functions = list_functions()
    functions = sort_functions(functions)

    print(f"Found {len(functions)} functions")

    jobs: list[tuple[str, str, list[str]]] = [
        # march,    target,         cpus
        ("x86-64", target_x86_64, cpus_x86_64),
        ("aarch64", target_aarch64, cpus_aarch64),
    ]

    for march, target, cpus in jobs:
        asms: dict[str, str] = {}

        # Generic asm for no specific cpu on this target
        for number, fn in functions:
            print(f"Extracting {march} asm for {fn}")

            asm = sanitize_asm(cargo_asm(number, target))

            asms[fn] = asm

        asm_filepath = make_asm_filepath(target, "generic")
        print(f"Writing {march} asm to {asm_filepath}")
        with asm_filepath.open("w", encoding="utf-8") as outfile:
            by_asm = transpose_asms(asms)
            for asm, fns in by_asm.items():
                outfile.write(f"## {format_fns(fns)}\n")
                outfile.write(f"```asm\n{asm}\n```\n")

        for cpu in cpus:
            # Asm for this specific cpu
            for number, fn in functions:
                print(f"Extracting {march} asm for {fn}")

                asm = sanitize_asm(cargo_asm(number, target, cpu))

                asms[fn] = asm

            print(f"Resolved {len(asms)} asms, including duplicates")

            asm_filepath = make_asm_filepath(target, cpu)
            print(f"Writing {march} asm to {asm_filepath}")
            with asm_filepath.open("w", encoding="utf-8") as outfile:
                by_asm = transpose_asms(asms)
                for asm, fns in by_asm.items():
                    outfile.write(f"## {format_fns(fns)}\n")
                    outfile.write(f"```asm\n{asm}\n```\n")

            with make_mca_filepath(cpu).open("w", encoding="utf-8") as outfile:
                instructions_legend: str = ""
                resources_legend: str = ""

                mcas: dict[str, list[str]] = {}
                by_asm = transpose_asms(asms)
                for asm, fns in by_asm.items():
                    print(f"Extracting mca for {fns} on {cpu}")

                    mca = run_llvm_mca(asm, march, cpu)

                    mca, instructions_legend, resources_legend = (
                        extract_llvm_mca_legends(mca)
                    )

                    mcas[mca] = fns

                outfile.write("# Instruction Info:\n")
                outfile.write(f"```\n{instructions_legend}\n```\n")

                outfile.write("# Resources:\n")
                outfile.write(f"```\n{resources_legend}\n```\n")

                outfile.write("# Functions:\n")

                for mca, fns in mcas.items():
                    outfile.write(f"## {format_fns(fns)}\n")
                    outfile.write(f"```asm\n{mca}\n```\n")


def main() -> None:
    """Detect the host, verify targets, and generate all asm/mca documents."""
    host_os = get_host_os()
    print(f"Detected OS: {host_os}")

    targets = get_target_triples(host_os)
    print(f"Expected targets: {targets}")

    try:
        installed = get_installed_targets()
    except FileNotFoundError:
        print("rustup not found in PATH")
        sys.exit(1)

    available, missing = check_targets(installed, targets)

    print("\nInstalled targets:")
    for t in installed:
        print(f"  - {t}")

    print("\nResults:")
    print("Available:")
    for t in available:
        print(f"  - {t}")

    print("Missing:")
    for t in missing:
        print(f"  - {t}")

    if missing:
        print("\nTo install missing targets:")
        for t in missing:
            print(f"  rustup target add {t}")

    if missing:
        return

    target_x86_64 = choose_x86_64_target(available)
    if target_x86_64 is None:
        print("Unexpectedly missing an x86_64 target")
        return

    target_aarch64 = choose_aarch64_target(available)
    if target_aarch64 is None:
        print("Unexpectedly missing an aarch64 target")
        return

    produce_docs(
        target_x86_64=target_x86_64,
        target_aarch64=target_aarch64,
        cpus_x86_64=CPUS_X86_64,
        cpus_aarch64=CPUS_AARCH64,
    )


if __name__ == "__main__":
    main()
