# Safety invariants

This crate wraps a handful of `unsafe` integer intrinsics in safe public APIs.
This document is the single source of truth for *why* each `unsafe` block is
sound. Every `unsafe` block in `src/` should map to one of the categories below;
if you add or change one, update this file.

## Type invariants

### `Pow2<T>` (where `T: UnsignedInt`)

Field: `exponent: u8`.

> **Invariant P1:** `exponent < T::BITS`.

It is established by every constructor and never violated afterwards:

- `Pow2::<T>::from_exponent(e)` returns `Err(Pow2OutOfRange)` when
  `e as u32 >= T::BITS`, so a `Pow2<T>` can only hold `e < T::BITS`.
- `align_of` / `align_of_val` go through `from_exponent`.
- The `VAL_*` / `KIBI` / … associated consts are only defined for widths where
  the exponent fits.
- `TryFrom<UnboundedPow2>` and `TryFrom<{integer}>` go through `from_exponent`.

`Pow2<T>` is `Copy` and has no interior mutability, so P1 cannot be broken after
construction.

### `UnboundedPow2`

Field: `exponent: u8`. There is **no** upper-bound invariant — it may hold any
`0..=255`. Whether it is usable with a given integer width is decided at each
call by:

> `UnboundedPow2::is_safe::<T>() == (exponent as u32 <= T::SAFE_SHIFT)`

where `SAFE_SHIFT` is `BITS - 1` for unsigned `T` and `BITS - 2` for signed `T`.

### `IntAtLeastAsWide<T>`

> **Invariant W1:** implemented for `L` only when `L::BITS >= T::BITS`.

The sealed impl list guarantees this; external code cannot add impls.

## Unsafe operations and their preconditions

All are inherent integer intrinsics re-exported through the `Int` trait:

| Operation | Precondition for soundness |
|-----------|----------------------------|
| `v.unchecked_shl(n)` / `v.unchecked_shr(n)` | `n < <type of v>::BITS` |
| `T::unchecked_mask(bits)` | `bits <= T::Unsigned::SAFE_SHIFT` (`= BITS - 1`) |
| `T::unchecked_highest_mask_bit(bits)` | `bits <= T::Unsigned::SAFE_SHIFT` |

`unchecked_mask` / `unchecked_highest_mask_bit` compute `1 << bits` (in the
unsigned domain) and must not shift out of range, hence the `BITS - 1` bound.

## Why every call site is sound

There are two families of `unsafe` call sites.

### 1. Operations with a `Pow2<T>` right-hand side

The shift/mask amount is always `rhs.exponent`, and the left-hand side is some
`L: IntAtLeastAsWide<T>` (often `L == T`). Then:

```
rhs.exponent < T::BITS        (Invariant P1)
T::BITS      <= L::BITS        (Invariant W1)
=> rhs.exponent < L::BITS      => unchecked_shl/shr on L is sound
=> rhs.exponent <= L::BITS - 1 = L::Unsigned::SAFE_SHIFT
                               => unchecked_mask/highest_mask_bit on L is sound
```

So P1 + W1 discharge every precondition. This covers `Div`/`Rem`/`Mul` by
`Pow2<T>`, `div_floor`, `rem_floor`, `div_ceil`, `div_round`,
`floor_to_multiple`, `ceil_to_multiple`, `round_to_multiple`, `is_multiple_of`,
`checked_*`, and `Pow2::value` / `Pow2::mask`.

### 2. The signed-division sign shift

The truncating signed `Div` computes `self.unchecked_shr(Self::BITS - 1)` to
extract the sign. `BITS - 1 < BITS`, so it is sound unconditionally.

## Operations that do NOT use `unsafe`

Everything with an `UnboundedPow2` right-hand side uses **checked or masked**
arithmetic (`mask`, `checked_shl`, `<<`/`>>` with a `debug_assert!(is_safe)`),
because `UnboundedPow2` carries no width invariant. The `unbounded_*` variants
handle out-of-range exponents explicitly and never rely on an unchecked shift.

## Validation

- **Miri** runs `cargo miri test --lib` in CI, executing the unchecked paths
  under `-Zmiri-strict-provenance`.
- The exhaustive reference-oracle tests drive every operation over all
  `u8/i8/u16/i16` values and every exponent, so any precondition violation that
  produced a wrong value would be caught.
