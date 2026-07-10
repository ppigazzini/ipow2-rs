[<img alt="GitHub" src="https://img.shields.io/badge/github-Sopel97/ipow2--rs-blue?logo=github" height="20">](https://github.com/Sopel97/ipow2-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/ipow2.svg?logo=rust" height="20">](https://crates.io/crates/ipow2)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ipow2-0?logo=docs.rs" height="20">](https://docs.rs/ipow2)

# ipow2 - Efficient power-of-two abstraction

This crate provides two complementary types for representing powers of two -
`Pow2<T>` for bounded exponents and `UnboundedPow2` for unbounded exponents -
together with a comprehensive suite of optimized arithmetic operations (division, modulo, rounding,
and alignment).

Aside from improved performance, such abstraction provides valuable guarantees.

## Installation

Run the following Cargo command in your project directory:

```shell
cargo add ipow2
```

Or add the following line to your Cargo.toml:

```toml
ipow2 = "0.2"
```

## Motivating Examples

### Memory address arithmetic

```rust
use ipow2::Pow2;
fn main() {
    // Any u16 power of two is also safe for operations on u64
    let page_size: Pow2<u16> = Pow2::from_exponent(12).unwrap(); // 4 KiB
    let addr: u64 = 0x1234_5004;
    assert_eq!(ipow2::floor_to_multiple(addr, page_size), 0x1234_5000);
    assert_eq!(ipow2::ceil_to_multiple(addr, page_size), 0x1234_6000);

    let large_addr: u64 = 0xFFFF_FFFF_FFFF_FF00;
    // Panics in debug due to overflow
    std::panic::catch_unwind(|| ipow2::ceil_to_multiple(large_addr, page_size));
    assert_eq!(ipow2::checked_ceil_to_multiple(large_addr, page_size), None);
}
```

### Chunking

```rust
use ipow2::Pow2;
fn main() {
    const CHUNK_SIZE: Pow2<u8> = Pow2::<u8>::VAL_16;
    let block_x: i32 = -17;
    let block_y: i32 = -4;
    assert_eq!(ipow2::floor_to_multiple(block_x, CHUNK_SIZE), -32);
    assert_eq!(ipow2::floor_to_multiple(block_y, CHUNK_SIZE), -16);
    assert_eq!(ipow2::div_floor(block_x, CHUNK_SIZE), -2);
    assert_eq!(ipow2::div_floor(block_y, CHUNK_SIZE), -1);
    assert_eq!(ipow2::rem_floor(block_x, CHUNK_SIZE), 15);
    assert_eq!(ipow2::rem_floor(block_y, CHUNK_SIZE), 12);
}
```

## Functionality

Aside from operators `*`, `*=`, `/`, `/=`, `%`, `%=` being overloaded the library
provides the following functions:

### Division variants

| Function    | Description                                                                         |
|-------------|-------------------------------------------------------------------------------------|
| `div_floor` | Floor division (round toward −∞). For unsigned integers this is identical to `/`.   |
| `rem_floor` | Remainder after floor division. Always non-negative for signed inputs (unlike `%`). |
| `div_ceil`  | Ceiling division (round toward +∞).                                                 |
| `div_round` | Rounding division (round half away from zero).                                      |

### Alignment / rounding to multiples

| Function            | Description                                                                                            |
|---------------------|--------------------------------------------------------------------------------------------------------|
| `floor_to_multiple` | Round `n` down to the nearest multiple of `pow2`.                                                      |
| `ceil_to_multiple`  | Round `n` up to the nearest multiple of `pow2`. Panics on overflow (debug) / wraps (release) like `+`. |
| `round_to_multiple` | Round `n` to the nearest multiple, half away from zero.                                                |
| `is_multiple_of`    | Returns `true` if `n` is an exact multiple of `pow2`.                                                  |

### Broader variants

`checked_*` and `unbounded_*` variants of all functions may be implemented for specific
power-of-two types, if their behavior is meaningfully different.

`checked_*` can be used as a safe alternative to otherwise panicking normal functions.

`unbounded_*` can be used if the `UnboundedPow2` can have exponents outside the
range supported by the left hand side operand.

```rust
use ipow2::{Pow2, Pow2OutOfRange, UnboundedPow2};
fn main() {
    let some_high_pow2 = UnboundedPow2::YOBI;
    let some_size: u64 = 555555555555;
    assert_eq!(Pow2::<u64>::try_from(some_high_pow2), Err(Pow2OutOfRange));
    // Panics in debug due to exponent being out of range
    std::panic::catch_unwind(|| ipow2::div_floor(some_size, some_high_pow2));
    assert_eq!(ipow2::unbounded_div_floor(some_size, some_high_pow2), 0);
}
```

## Performance notes

- All operations are marked `#[inline(always)]` and generate small (up to ~20 instructions) assembly code.
- Operations on the safe `Pow2<T>` exploit `unchecked_shl` / `unchecked_shr`, which removes
  the shift-amount masking that Rust normally inserts for `u8` and `u16` operands.
- Implementations that widen the LHS operand are **NOT** used, facilitating effective vectorization.
- All functions other than `checked_*` and `unbounded_*` are **branchless**.

## Additional resources

See [docs README](./docs/README.md)
