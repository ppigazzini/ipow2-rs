# Docs

For code documentation `cargo doc` is recommended. The crate-level documentation
provides some usage examples.

This document provides information on some additional resources,
like assembly listings, `llvm-mca` analysis, and benchmarks.

## Assembly analysis

This repository provides assembly listings and `llvm-mca` analysis
for various targets for all functions from this crate.

The listings are generated using the [make_asm_docs.py](../scripts/make_asm_docs.py) script.
The script uses https://github.com/pacak/cargo-show-asm for disassembly.

Note that it requires nightly versions of the rust compiler due to required flags not being
in stable yet. On Windows the AArch64 targets require installing the targets in rustup
as well as VS Build tools for ARM64.

### Assembly

For the purpose of presentation the function names are encoded as follows:

`{function_name}_{type}_{rhs}` where

- `function_name` matches the free function names in this crate
- `type` is any of `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`
- `rhs` is either `pow2` or `unb_pow2`, corresponding to either `Pow2<T>` or `UnboundedPow2` respectively

Assembly listings for all functions can be found in the following locations in this folder:

- [x86-64 generic](asm-x86_64-pc-windows-msvc-generic.md)
- [x86-64 Intel Raptor Lake](asm-x86_64-pc-windows-msvc-raptorlake.md)
- [x86-64 AMD Zen 4](asm-x86_64-pc-windows-msvc-znver4.md)
- [AArch64 generic](asm-aarch64-pc-windows-msvc-generic.md)
- [AArch64 Apple M1](asm-aarch64-pc-windows-msvc-apple-m1.md)

### llvm-mca

Due to the difficulty of avoiding loop-carried dependencies when generating these listings
automatically from disassembly listings, they are restricted to a single iteration.
Because of this they don't provide full information on throughput, just latency, although
some estimation is present via the `RThroughput` value (theoretical best achievable cycles per block).

See [this section](#public-benchmarks) for complementary benchmarks that address throughput.

These listings can be found in the following locations in this folder:

- [x86-64 Intel Raptor Lake](mca-raptorlake.md)
- [x86-64 AMD Zen 4](mca-znver4.md)
- [AArch64 Apple M1](mca-apple-m1.md)

## Public Benchmarks

This repository provides some benchmarks for the basic variants of functions from this crate.

These benchmarks are defined in [this file](../benches/public.rs) and the runner is present
[here](../scripts/make_bench_docs.py)

Due to the difficulty of microbenchmarking such tiny functions the benchmarks focus
on vectorizable workloads. Each benchmark case is a mapping of a 1024 input array
into a 1024 output array.

These cases are further subdivided by two criteria:

- right hand side type
    - `Pow2<T>` (infix `pow2`)
    - `UnboundedPow2` (infix `unb_pow2`)
- right hand side "constness"
  - RHS is unique for every input (no suffix)
  - RHS is a runtime constant reused for every input (suffix `_reuse`)
  - RHS is a compile-time constant reused for every input (suffix `_const`)

The benchmark name is then defined as follows:

`{function_name}_{infix}{suffix}`

Additionally, cases with `std_` are provided to show performance of explicit division/multiplication.

Additional `baseline_identity` case is provided which is basically `memcpy`.

Since each input has 1024 elements the time listings report time required to process 1024 elements.

Some benchmark runs can be found in the following locations in this folder:

- [x86-64 generic](bench-default.md)
- [x86-64 native (AMD Zen 4)](bench-native.md)
