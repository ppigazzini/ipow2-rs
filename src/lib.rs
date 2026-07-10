//! [<img alt="GitHub" src="https://img.shields.io/badge/github-Sopel97/ipow2--rs-blue?logo=github" height="20">](https://github.com/Sopel97/ipow2-rs)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/ipow2.svg?logo=rust" height="20">](https://crates.io/crates/ipow2)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ipow2-0?logo=docs.rs" height="20">](https://docs.rs/ipow2)
//!
//! # ipow2 - Efficient power-of-two abstraction
//!
//! This crate provides two complementary types for representing powers of two, together
//! with a comprehensive suite of arithmetic operations (division, modulo, rounding, and
//! alignment) that are optimized to exploit the fact that the divisor is a power of two.
//! Aside from improved performance, such abstraction provides valuable guarantees.
//!
//! ## Core types
//!
//! ### [`Pow2<T>`]
//!
//! The preferred type. The type parameter `T` must be an unsigned primitive integer
//! (`u8`, `u16`, `u32`, `u64`, `u128`, or `usize`), and the stored exponent is
//! guaranteed to be a valid shift amount for `T` - that is, strictly less than
//! `T::BITS`. Note that while the generic type is always unsigned it is only
//! used to signify the bit-width of the desired type. [`Pow2<T>`] can be used
//! in operations with signed left hand sides.
//!
//! This guarantee, aside from being safer in general, lets the implementation
//! use `unchecked_shl` / `unchecked_shr` in safe contexts, avoiding the masking that
//! Rust's wrapping shift semantics would otherwise require for types narrower than 32 bits.
//!
//! ```
//! use ipow2::Pow2;
//! // Any u16 power of two is also safe for operations on u64
//! let page_size: Pow2<u16> = Pow2::from_exponent(12).unwrap(); // 4 KiB
//! let addr: u64 = 0x1234_5004;
//! assert_eq!(ipow2::floor_to_multiple(addr, page_size), 0x1234_5000);
//! assert_eq!(ipow2::ceil_to_multiple(addr, page_size), 0x1234_6000);
//!
//! let large_addr: u64 = 0xFFFF_FFFF_FFFF_FF00;
//! // Panics in debug due to overflow
//! std::panic::catch_unwind(|| ipow2::ceil_to_multiple(large_addr, page_size));
//! assert_eq!(ipow2::checked_ceil_to_multiple(large_addr, page_size), None);
//! ```
//!
//! ```
//! use ipow2::Pow2;
//! const CHUNK_SIZE: Pow2<u8> = Pow2::<u8>::VAL_16;
//! let block_x: i32 = -17;
//! let block_y: i32 = -4;
//! assert_eq!(ipow2::floor_to_multiple(block_x, CHUNK_SIZE), -32);
//! assert_eq!(ipow2::floor_to_multiple(block_y, CHUNK_SIZE), -16);
//! assert_eq!(ipow2::div_floor(block_x, CHUNK_SIZE), -2);
//! assert_eq!(ipow2::div_floor(block_y, CHUNK_SIZE), -1);
//! assert_eq!(ipow2::rem_floor(block_x, CHUNK_SIZE), 15);
//! assert_eq!(ipow2::rem_floor(block_y, CHUNK_SIZE), 12);
//! ```
//!
//! ### [`UnboundedPow2`]
//!
//! A simpler type that models and power-of-two with an exponent representable as `u8`,
//! covering the range of exponents `0..=255`. Because the exponent is not constrained to any integer width,
//! operations that depend on the shift being in range must either use the `checked_*`
//! variants or assert safety themselves. Prefer [`Pow2<T>`] when the target integer
//! type is known. However, it may be needed in extreme cases, for example unbounded operations:
//!
//! ```
//! use ipow2::{Pow2, Pow2OutOfRange, UnboundedPow2};
//! let some_high_pow2 = UnboundedPow2::YOBI;
//! let some_size: u64 = 555555555555;
//! assert_eq!(Pow2::<u64>::try_from(some_high_pow2), Err(Pow2OutOfRange));
//! // Panics in debug due to exponent being out of range
//! std::panic::catch_unwind(|| ipow2::div_floor(some_size, some_high_pow2));
//! assert_eq!(ipow2::unbounded_div_floor(some_size, some_high_pow2), 0);
//! ```
//!
//! Both types implement `Copy`, `Clone`, `Debug`, `Hash`, `Eq`, `PartialEq`, `Ord`, and
//! `PartialOrd`, and are guaranteed to have the same size and alignment as `u8`.
//!
//! ## Predefined constants
//!
//! Common powers of two are provided as associated constants on both types:
//!
//! | Constant  | Value |
//! |-----------|-------|
//! | `VAL_1` … `VAL_512` | 1, 2, 4, … 512 |
//! | `KIBI`    | 2¹⁰ = 1 024 |
//! | `MEBI`    | 2²⁰ = 1 048 576 |
//! | `GIBI`    | 2³⁰ |
//! | `TEBI`    | 2⁴⁰ |
//! | … up to `QUEBI` | 2¹⁰⁰ (on `UnboundedPow2` / wide `Pow2` types) |
//!
//! `Pow2<usize>` exposes only the constants that fit on the current target pointer width
//! (up to `GIBI` on 32-bit, up to `EXBI` on 64-bit).
//!
//! ## Operator overloads on integers
//!
//! All primitive integer types (`i8` … `i128`, `u8` … `u128`, `isize`, `usize`)
//! implement `Mul`, `MulAssign`, `Div`, `DivAssign`, `Rem`, and `RemAssign`
//! against both [`Pow2<T>`] and [`UnboundedPow2`]. The follow the standard semantics,
//! so division rounds towards zero.
//!
//! ## Free functions and their dispatch traits
//!
//! Every operation is exposed as a free function that dispatches through a corresponding
//! trait. The trait names are PascalCase versions of the function names, e.g.
//! [`DivFloor`] / [`div_floor`]. This design lets you use the functions in generic
//! code by bounding on the trait, or call them directly on concrete types.
//!
//! ### Division variants
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`div_floor`] | Floor division (round toward −∞). For unsigned integers this is identical to `/`. |
//! | [`rem_floor`] | Remainder after floor division. Always non-negative for signed inputs (unlike `%`). |
//! | [`div_ceil`] | Ceiling division (round toward +∞). |
//! | [`div_round`] | Rounding division (round half away from zero). |
//!
//! ### Alignment / rounding to multiples
//!
//! | Function | Description |
//! |----------|-------------|
//! | [`floor_to_multiple`] | Round `n` down to the nearest multiple of `pow2`. |
//! | [`ceil_to_multiple`] | Round `n` up to the nearest multiple of `pow2`. Panics on overflow (debug) / wraps (release) like `+`. |
//! | [`round_to_multiple`] | Round `n` to the nearest multiple, half away from zero. |
//! | [`is_multiple_of`] | Returns `true` if `n` is an exact multiple of `pow2`. |
//!
//! ### Additional variants
//!
//! `checked_*` and `unbounded_*` variants of all functions may be implemented for specific
//! power-of-two types, if their behavior is meaningfully different.
//!
//! `checked_*` can be used as a safe alternative to otherwise panicking normal functions.
//!
//! `unbounded_*` can be used if the [`UnboundedPow2`] can have exponents outside the
//! range supported by the left hand side operand.
//!
//! ## Type conversions
//!
//! - [`Pow2<T>`] converts infallibly to [`UnboundedPow2`] via [`From`].
//! - [`UnboundedPow2`] converts to [`Pow2<T>`] via [`TryFrom`], returning
//!   [`Pow2OutOfRange`] if the exponent does not fit.
//! - [`Pow2<T>`] converts to a wider [`Pow2<U>`] (where `U` is at least as wide as `T`)
//!   via [`From`]. Narrowing conversions use [`TryFrom`].
//! - Both types can be constructed from integers via [`TryFrom`]
//! - Both types can be converted to integers, either via [`Pow2::value`] or
//!   [`UnboundedPow2::as_u8`] and similar associated functions.
//!
//! ## Type aliases
//!
//! For convenience, the following type aliases are re-exported:
//!
//! ```ignore
//! type Pow2<T>u8    = Pow2<u8>;
//! type Pow2<T>u16   = Pow2<u16>;
//! type Pow2<T>u32   = Pow2<u32>;
//! type Pow2<T>u64   = Pow2<u64>;
//! type Pow2<T>u128  = Pow2<u128>;
//! type Pow2<T>usize = Pow2<usize>;
//! ```
//!
//! ## Integer traits
//!
//! The crate re-exports four traits from its private implementation module that you can
//! use as generic bounds (other use is highly discouraged):
//!
//! - [`Int`] - any primitive integer (signed or unsigned).
//! - [`UnsignedInt`] - any unsigned primitive integer.
//! - [`SignedInt`] - any signed primitive integer.
//! - [`IntAtLeastAsWide<T>`] - an integer whose bit-width is >= that of `T`.
//!
//! These are sealed traits; you cannot implement them for your own types.
//!
//! ## Choosing between `Pow2<T>` and `UnboundedPow2`
//!
//! | Situation | Recommendation |
//! |-----------|----------------|
//! | Target integer type is known at compile time | [`Pow2<T>`] - stronger guarantees, fewer runtime checks, enables `unchecked` shifts |
//! | The exponent is in range `0..=7` | [`Pow2<u8>`] - same as above, valid for operations on all integers |
//! | Exponent may exceed the target integer width (e.g., a "virtual" power of two used only for comparison or serialization or with `unbounded_*` functions) | [`UnboundedPow2`] |
//! | Storing in a collection alongside integers of mixed widths | [`UnboundedPow2`] |
//!
//! ## Performance notes
//!
//! - [`Pow2<T>`] and [`UnboundedPow2`] are both `repr(transparent)` over `u8`, so they
//!   have zero overhead as function arguments or struct fields.
//! - All operations are marked `#[inline(always)]` and generate small (up to ~20 instructions) assembly code.
//! - Operations on [`Pow2<T>`] exploit `unchecked_shl` / `unchecked_shr`, which removes
//!   the shift-amount masking that Rust normally inserts for `u8` and `u16` operands.
//! - Many functions can reuse partial results from previous calls or other functions
//!   when using the same RHS operand repeatedly. In some cases such implementations
//!   are favored over alternatives that would produce slightly lower latency.
//!   The rationale is that these functions are expected to be used on
//!   higher-dimensional (>=2) data (where there are multiple coordinates to process)
//! - Implementations that widen the LHS operand are **NOT** used,
//!   facilitating effective vectorization with both unique and reused RHS values.
//! - All functions other than `checked_*` and `unbounded_*` are **branchless**.
//! - For scalar execution the differences between default CPU targets and modern CPU targets
//!   are minor. However, if good vectorization is desired it is strongly recommended to
//!   enable at least AVX2 on x86-64 as SSE has limited support for bitwise operations.
//!
//! ## Documentation notes
//!
//! Due to lack of ability to link to specific trait implementations from source docs
//! this crate uses an unconventional documentation pattern. See [`__detached_docs`]
//! for documentations on specific trait implementations.

use std::error::Error;
use std::ops::{Div, DivAssign, Mul, MulAssign, Rem, RemAssign};
use std::{cmp, marker};

#[macro_use]
mod private;

// Expose the private Int trait as API to allow users to specify generic bounds.
pub use crate::private::{Int, IntAtLeastAsWide, SignedInt, UnsignedInt};

/// Models a power of two integer by storing the exponent.
/// The exponent is stored as a `u8` so the available values are 0..=255.
///
/// It is strongly recommended to use [`Pow2`] whenever possible instead due to
/// stronger guarantees and higher performance in some cases.
///
/// Some operations involving [`UnboundedPow2`] are faster on unsigned integers,
/// so they should be preferred, unless required otherwise.
#[repr(transparent)]
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct UnboundedPow2 {
    exponent: u8,
}

const _: () = assert!(size_of::<UnboundedPow2>() == size_of::<u8>());
const _: () = assert!(align_of::<UnboundedPow2>() == align_of::<u8>());

/// Error type for when a value is not a power of two.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct NotPow2;

impl Error for NotPow2 {}
impl std::fmt::Display for NotPow2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Not a power of two")
    }
}

/// Error type for when a power of two value cannot be represented.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Pow2OutOfRange;

impl Error for Pow2OutOfRange {}
impl std::fmt::Display for Pow2OutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Out of range")
    }
}

#[rustfmt::skip]
impl UnboundedPow2 {
    pub const VAL_1: Self = Self::from_exponent(0);
    pub const VAL_2: Self = Self::from_exponent(1);
    pub const VAL_4: Self = Self::from_exponent(2);
    pub const VAL_8: Self = Self::from_exponent(3);
    pub const VAL_16: Self = Self::from_exponent(4);
    pub const VAL_32: Self = Self::from_exponent(5);
    pub const VAL_64: Self = Self::from_exponent(6);
    pub const VAL_128: Self = Self::from_exponent(7);
    pub const VAL_256: Self = Self::from_exponent(8);
    pub const VAL_512: Self = Self::from_exponent(9);

    pub const KIBI: Self = Self::from_exponent(10);
    pub const MEBI: Self = Self::from_exponent(20);
    pub const GIBI: Self = Self::from_exponent(30);
    pub const TEBI: Self = Self::from_exponent(40);
    pub const PEBI: Self = Self::from_exponent(50);
    pub const EXBI: Self = Self::from_exponent(60);
    pub const ZEBI: Self = Self::from_exponent(70);
    pub const YOBI: Self = Self::from_exponent(80);
    pub const ROBI: Self = Self::from_exponent(90);
    pub const QUEBI: Self = Self::from_exponent(100);
}

impl UnboundedPow2 {
    /// Creates a new [`UnboundedPow2`] from a given power of two exponent.
    #[must_use]
    #[inline(always)]
    pub const fn from_exponent(exponent: u8) -> UnboundedPow2 {
        UnboundedPow2 { exponent }
    }

    /// Creates a new [`UnboundedPow2`] equal to the memory alignment of type `T`
    #[must_use]
    #[inline(always)]
    pub const fn align_of<T>() -> UnboundedPow2 {
        UnboundedPow2 {
            exponent: align_of::<T>().ilog2() as u8,
        }
    }

    /// Creates a new [`UnboundedPow2`] equal to the memory alignment of value `val`
    #[must_use]
    #[inline(always)]
    pub const fn align_of_val<T: ?Sized>(val: &T) -> UnboundedPow2 {
        UnboundedPow2 {
            exponent: align_of_val(val).ilog2() as u8,
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn exponent(self) -> u8 {
        self.exponent
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_safe<T: Int>(self) -> bool {
        self.exponent as u32 <= T::SAFE_SHIFT
    }

    /// Attempts to multiply two [`UnboundedPow2`] together, producing a new [`UnboundedPow2`].
    ///
    /// Returns `None` if the result exponent is not representable, `Some(result)` otherwise.
    #[must_use]
    #[inline(always)]
    pub fn checked_mul(self, other: UnboundedPow2) -> Option<UnboundedPow2> {
        Some(UnboundedPow2::from_exponent(
            self.exponent.checked_add(other.exponent)?,
        ))
    }

    /// Multiplies two [`UnboundedPow2`] with saturation together, producing a new [`UnboundedPow2`].
    /// Effectively saturated addition of exponents.
    #[must_use]
    #[inline(always)]
    pub const fn saturating_mul(self, other: UnboundedPow2) -> UnboundedPow2 {
        UnboundedPow2::from_exponent(self.exponent.saturating_add(other.exponent))
    }

    /// Attempts to divide two [`UnboundedPow2`] together, producing a new [`UnboundedPow2`].
    ///
    /// Returns `None` if the result exponent is not representable, `Some(result)` otherwise.
    #[must_use]
    #[inline(always)]
    pub fn checked_div(self, other: UnboundedPow2) -> Option<UnboundedPow2> {
        Some(UnboundedPow2::from_exponent(
            self.exponent.checked_sub(other.exponent)?,
        ))
    }

    /// Divides two [`UnboundedPow2`] with saturation together, producing a new [`UnboundedPow2`].
    /// Effectively saturated division of exponents.
    #[must_use]
    #[inline(always)]
    pub const fn saturating_div(self, other: UnboundedPow2) -> UnboundedPow2 {
        UnboundedPow2::from_exponent(self.exponent.saturating_sub(other.exponent))
    }
}

macro_rules! impl_as {
    ($name:ident, $t:ty) => {
        impl UnboundedPow2 {
            #[doc = concat!(
                                "Returns the modelled power-of-two value as ", stringify!($t), "."
                            )]
            /// # Panics
            ///
            /// In debug builds, panics if the value is not representable.
            /// In release builds this check is skipped; the result will be defined but incorrect.
            #[must_use]
            #[inline(always)]
            pub fn $name(self) -> $t {
                debug_assert!(self.is_safe::<$t>());
                1 << self.exponent
            }
        }
    };
}

impl_as!(as_u8, u8);
impl_as!(as_u16, u16);
impl_as!(as_u32, u32);
impl_as!(as_u64, u64);
impl_as!(as_u128, u128);
impl_as!(as_usize, usize);

impl_as!(as_i8, i8);
impl_as!(as_i16, i16);
impl_as!(as_i32, i32);
impl_as!(as_i64, i64);
impl_as!(as_i128, i128);
impl_as!(as_isize, isize);

macro_rules! impl_try_from {
    ($t:ty) => {
        impl TryFrom<$t> for UnboundedPow2 {
            type Error = NotPow2;

            #[inline(always)]
            fn try_from(value: $t) -> Result<Self, Self::Error> {
                if !value.is_power_of_two() {
                    Err(NotPow2)
                } else {
                    Ok(UnboundedPow2::from_exponent(value.ilog2() as u8))
                }
            }
        }

        impl TryFrom<UnboundedPow2> for $t {
            type Error = Pow2OutOfRange;

            #[inline(always)]
            fn try_from(value: UnboundedPow2) -> Result<Self, Self::Error> {
                if value.is_safe::<$t>() {
                    Ok(1 << value.exponent)
                } else {
                    Err(Pow2OutOfRange)
                }
            }
        }
    };
}

impl_try_from!(u8);
impl_try_from!(u16);
impl_try_from!(u32);
impl_try_from!(u64);
impl_try_from!(u128);
impl_try_from!(usize);

impl_try_from!(i8);
impl_try_from!(i16);
impl_try_from!(i32);
impl_try_from!(i64);
impl_try_from!(i128);
impl_try_from!(isize);

impl Mul for UnboundedPow2 {
    type Output = UnboundedPow2;

    #[allow(clippy::suspicious_arithmetic_impl)]
    #[inline(always)]
    fn mul(self, other: UnboundedPow2) -> UnboundedPow2 {
        UnboundedPow2::from_exponent(self.exponent + other.exponent)
    }
}

impl MulAssign for UnboundedPow2 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn mul_assign(&mut self, other: UnboundedPow2) {
        self.exponent += other.exponent;
    }
}

impl Div for UnboundedPow2 {
    type Output = UnboundedPow2;

    #[inline(always)]
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, other: UnboundedPow2) -> UnboundedPow2 {
        UnboundedPow2::from_exponent(self.exponent - other.exponent)
    }
}

impl DivAssign for UnboundedPow2 {
    #[inline(always)]
    #[allow(clippy::suspicious_op_assign_impl)]
    fn div_assign(&mut self, other: UnboundedPow2) {
        self.exponent -= other.exponent;
    }
}

/// Models a power of two integer by storing the exponent.
/// The exponent is guaranteed to be in the range valid for bitshift operations
/// on `T`. `T` is restricted to unsigned integers because it's mostly used for modeling bit-width.
///
/// It is strongly recommended to use this struct instead of [`UnboundedPow2`] due to
/// stronger guarantees and higher performance in some cases. Notably:
/// - more operations are guaranteed to return a correct result for all inputs
/// - operations on integers smaller than 32-bits may omit masking shift amount that's required for wrapping semantics
/// - some unnecessary checks for too high exponent may be omitted
///
/// Some operations involving [`Pow2`] are faster on unsigned integers,
/// so they should be preferred, unless required otherwise.
#[repr(transparent)]
#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct Pow2<T> {
    exponent: u8,
    _marker: marker::PhantomData<T>,
}

const _: () = assert!(size_of::<Pow2<u128>>() == size_of::<u8>());
const _: () = assert!(align_of::<Pow2<u128>>() == align_of::<u8>());

impl<T> Pow2<T>
where
    T: UnsignedInt,
{
    /// Attempts to create a new [`Pow2`] from a given power of two exponent.
    ///
    /// Returns `Err(Pow2OutOfRange)` if the exponent is too high for this integer width,
    /// `Ok(result)` otherwise.
    #[inline(always)]
    pub const fn from_exponent(exponent: u8) -> Result<Self, Pow2OutOfRange> {
        if exponent as u32 >= T::BITS {
            Err(Pow2OutOfRange)
        } else {
            Ok(Self {
                exponent,
                _marker: marker::PhantomData,
            })
        }
    }

    /// Attempts to create a new [`Pow2`] equal to the memory alignment of type `T`.
    ///
    /// Returns `Err(Pow2OutOfRange)` if the exponent is too high for this integer width,
    /// `Ok(result)` otherwise.
    #[inline(always)]
    pub const fn align_of<U>() -> Result<Self, Pow2OutOfRange> {
        Self::from_exponent(align_of::<U>().ilog2() as u8)
    }

    /// Attempts to create a new [`Pow2`] equal to the memory alignment of value `val`.
    ///
    /// Returns `Err(Pow2OutOfRange)` if the exponent is too high for this integer width,
    /// `Ok(result)` otherwise.
    #[inline(always)]
    pub const fn align_of_val<U: ?Sized>(val: &U) -> Result<Self, Pow2OutOfRange> {
        Self::from_exponent(align_of_val(val).ilog2() as u8)
    }

    #[must_use]
    #[inline(always)]
    pub const fn exponent(self) -> u8 {
        self.exponent
    }

    /// Returns the actual value modeled by this object.
    #[must_use]
    #[inline(always)]
    pub fn value(self) -> T {
        // SAFETY: Pow2<T> guarantees a valid shift
        unsafe { T::ONE.unchecked_shl(self.exponent as u32) }
    }

    /// Returns a mask with `self.exponent` of lowest bits set. Effectively `self.value() - 1`.
    #[must_use]
    #[inline(always)]
    pub fn mask(self) -> T {
        // SAFETY: Pow2<T> guarantees a valid shift
        unsafe { T::unchecked_mask(self.exponent as u32) }
    }

    /// Attempts to multiply two [`Pow2`] together, producing a new [`Pow2`].
    ///
    /// Returns `None` if the result exponent is too high for this integer width, `Some(result)` otherwise.
    #[must_use]
    #[inline(always)]
    pub fn checked_mul(self, other: Self) -> Option<Self> {
        // The addition does not overflow because it's at most 127+127<=255 for u128
        Self::from_exponent(self.exponent + other.exponent).ok()
    }

    /// Multiplies two [`Pow2`] with saturation together, producing a new [`Pow2`].
    /// Effectively saturated addition of exponents.
    ///
    /// NOTE: The saturation upper bound is defined by the current integer width.
    #[must_use]
    #[inline(always)]
    pub fn saturating_mul(self, other: Self) -> Self {
        Self {
            exponent: cmp::min(self.exponent + other.exponent, (T::BITS - 1) as u8),
            _marker: marker::PhantomData,
        }
    }

    /// Attempts to divide two [`Pow2`] together, producing a new [`Pow2`].
    ///
    /// Returns `None` if the result exponent is too high for this integer width, `Some(result)` otherwise.
    #[must_use]
    #[inline(always)]
    pub fn checked_div(self, other: Self) -> Option<Self> {
        Some(Self {
            exponent: self.exponent.checked_sub(other.exponent)?,
            _marker: marker::PhantomData,
        })
    }

    /// Divides two [`Pow2`] with saturation together, producing a new [`Pow2`].
    /// Effectively saturated division of exponents.
    #[must_use]
    #[inline(always)]
    pub const fn saturating_div(self, other: Self) -> Self {
        Self {
            exponent: self.exponent.saturating_sub(other.exponent),
            _marker: marker::PhantomData,
        }
    }
}

#[rustfmt::skip]
impl Pow2<u8> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
}

#[rustfmt::skip]
impl Pow2<u16> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
}

#[rustfmt::skip]
impl Pow2<u32> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
    pub const MEBI   : Self = Self { exponent: 20,  _marker: marker::PhantomData };
    pub const GIBI   : Self = Self { exponent: 30,  _marker: marker::PhantomData };
}

#[rustfmt::skip]
impl Pow2<u64> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
    pub const MEBI   : Self = Self { exponent: 20,  _marker: marker::PhantomData };
    pub const GIBI   : Self = Self { exponent: 30,  _marker: marker::PhantomData };
    pub const TEBI   : Self = Self { exponent: 40,  _marker: marker::PhantomData };
    pub const PEBI   : Self = Self { exponent: 50,  _marker: marker::PhantomData };
    pub const EXBI   : Self = Self { exponent: 60,  _marker: marker::PhantomData };
}

#[rustfmt::skip]
impl Pow2<u128> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
    pub const MEBI   : Self = Self { exponent: 20,  _marker: marker::PhantomData };
    pub const GIBI   : Self = Self { exponent: 30,  _marker: marker::PhantomData };
    pub const TEBI   : Self = Self { exponent: 40,  _marker: marker::PhantomData };
    pub const PEBI   : Self = Self { exponent: 50,  _marker: marker::PhantomData };
    pub const EXBI   : Self = Self { exponent: 60,  _marker: marker::PhantomData };
    pub const ZEBI   : Self = Self { exponent: 70,  _marker: marker::PhantomData };
    pub const YOBI   : Self = Self { exponent: 80,  _marker: marker::PhantomData };
    pub const ROBI   : Self = Self { exponent: 90,  _marker: marker::PhantomData };
    pub const QUEBI  : Self = Self { exponent: 100, _marker: marker::PhantomData };
}

#[rustfmt::skip]
#[cfg(target_pointer_width = "16")]
impl Pow2<usize> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
}

#[rustfmt::skip]
#[cfg(target_pointer_width = "32")]
impl Pow2<usize> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
    pub const MEBI   : Self = Self { exponent: 20,  _marker: marker::PhantomData };
    pub const GIBI   : Self = Self { exponent: 30,  _marker: marker::PhantomData };
}

#[rustfmt::skip]
#[cfg(target_pointer_width = "64")]
impl Pow2<usize> {
    pub const VAL_1  : Self = Self { exponent: 0,   _marker: marker::PhantomData };
    pub const VAL_2  : Self = Self { exponent: 1,   _marker: marker::PhantomData };
    pub const VAL_4  : Self = Self { exponent: 2,   _marker: marker::PhantomData };
    pub const VAL_8  : Self = Self { exponent: 3,   _marker: marker::PhantomData };
    pub const VAL_16 : Self = Self { exponent: 4,   _marker: marker::PhantomData };
    pub const VAL_32 : Self = Self { exponent: 5,   _marker: marker::PhantomData };
    pub const VAL_64 : Self = Self { exponent: 6,   _marker: marker::PhantomData };
    pub const VAL_128: Self = Self { exponent: 7,   _marker: marker::PhantomData };
    pub const VAL_256: Self = Self { exponent: 8,   _marker: marker::PhantomData };
    pub const VAL_512: Self = Self { exponent: 9,   _marker: marker::PhantomData };

    pub const KIBI   : Self = Self { exponent: 10,  _marker: marker::PhantomData };
    pub const MEBI   : Self = Self { exponent: 20,  _marker: marker::PhantomData };
    pub const GIBI   : Self = Self { exponent: 30,  _marker: marker::PhantomData };
    pub const TEBI   : Self = Self { exponent: 40,  _marker: marker::PhantomData };
    pub const PEBI   : Self = Self { exponent: 50,  _marker: marker::PhantomData };
    pub const EXBI   : Self = Self { exponent: 60,  _marker: marker::PhantomData };
}

impl<T> From<Pow2<T>> for UnboundedPow2 {
    #[inline(always)]
    fn from(value: Pow2<T>) -> Self {
        Self {
            exponent: value.exponent,
        }
    }
}

impl<T> TryFrom<UnboundedPow2> for Pow2<T>
where
    T: UnsignedInt,
{
    type Error = Pow2OutOfRange;

    #[inline(always)]
    fn try_from(value: UnboundedPow2) -> Result<Self, Self::Error> {
        Self::from_exponent(value.exponent)
    }
}

/// Error type for when a Pow2 cannot be constructed from an integer.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Pow2TryFromIntError {
    Pow2OutOfRange,
    NotPow2,
}

impl Error for Pow2TryFromIntError {}
impl std::fmt::Display for Pow2TryFromIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Pow2TryFromIntError::Pow2OutOfRange => {
                write!(f, "Out of range")
            }
            Pow2TryFromIntError::NotPow2 => {
                write!(f, "Not a power of two")
            }
        }
    }
}

impl From<NotPow2> for Pow2TryFromIntError {
    fn from(_: NotPow2) -> Self {
        Pow2TryFromIntError::NotPow2
    }
}

impl From<Pow2OutOfRange> for Pow2TryFromIntError {
    fn from(_: Pow2OutOfRange) -> Self {
        Pow2TryFromIntError::Pow2OutOfRange
    }
}

macro_rules! impl_pow2_try_from_int {
    ($t:ty) => {
        impl<T> TryFrom<$t> for Pow2<T>
        where
            T: UnsignedInt,
        {
            type Error = Pow2TryFromIntError;

            #[inline(always)]
            fn try_from(value: $t) -> Result<Self, Self::Error> {
                if !value.is_power_of_two() {
                    Err(Pow2TryFromIntError::NotPow2)
                } else {
                    Pow2::from_exponent(value.ilog2() as u8).map_err(|e| e.into())
                }
            }
        }
    };
}

impl_pow2_try_from_int!(u8);
impl_pow2_try_from_int!(u16);
impl_pow2_try_from_int!(u32);
impl_pow2_try_from_int!(u64);
impl_pow2_try_from_int!(u128);
impl_pow2_try_from_int!(usize);
impl_pow2_try_from_int!(i8);
impl_pow2_try_from_int!(i16);
impl_pow2_try_from_int!(i32);
impl_pow2_try_from_int!(i64);
impl_pow2_try_from_int!(i128);
impl_pow2_try_from_int!(isize);

macro_rules! impl_pow2_self_conv_from {
    ($t:ty => $($into:ty),*) => {
        $(
            impl From<Pow2<$t>> for Pow2<$into> {
                #[inline(always)]
                fn from(value: Pow2<$t>) -> Self {
                    Self {
                        exponent: value.exponent,
                        _marker: marker::PhantomData,
                    }
                }
            }
        )*
    };
}

impl_pow2_self_conv_from!(u8 => u16, u32, u64, u128, usize);

#[cfg(target_pointer_width = "16")]
impl_pow2_self_conv_from!(u16 => u32, u64, u128);

#[cfg(target_pointer_width = "32")]
impl_pow2_self_conv_from!(u16 => u32, u64, u128, usize);

#[cfg(target_pointer_width = "64")]
impl_pow2_self_conv_from!(u16 => u32, u64, u128, usize);

#[cfg(target_pointer_width = "16")]
impl_pow2_self_conv_from!(u32 => u64, u128);

#[cfg(target_pointer_width = "32")]
impl_pow2_self_conv_from!(u32 => u64, u128);

#[cfg(target_pointer_width = "64")]
impl_pow2_self_conv_from!(u32 => u64, u128, usize);

impl_pow2_self_conv_from!(u64 => u128);

macro_rules! impl_pow2_self_conv_try_from {
    ($t:ty => $($into:ty),*) => {
        $(
            impl TryFrom<Pow2<$t>> for Pow2<$into> {
                type Error = Pow2OutOfRange;

                #[inline(always)]
                fn try_from(value: Pow2<$t>) -> Result<Self, Self::Error> {
                    Pow2::from_exponent(value.exponent)
                }
            }
        )*
    };
}

impl_pow2_self_conv_try_from!(u16 => u8);

#[cfg(target_pointer_width = "16")]
impl_pow2_self_conv_try_from!(u32 => u8, u16, usize);

#[cfg(target_pointer_width = "32")]
impl_pow2_self_conv_try_from!(u32 => u8, u16);

#[cfg(target_pointer_width = "64")]
impl_pow2_self_conv_try_from!(u32 => u8, u16);

#[cfg(target_pointer_width = "16")]
impl_pow2_self_conv_try_from!(u64 => u8, u16, u32, usize);

#[cfg(target_pointer_width = "32")]
impl_pow2_self_conv_try_from!(u64 => u8, u16, u32, usize);

#[cfg(target_pointer_width = "64")]
impl_pow2_self_conv_try_from!(u64 => u8, u16, u32);

impl_pow2_self_conv_try_from!(u128 => u8, u16, u32, u64);

pub type Pow2u8 = Pow2<u8>;
pub type Pow2u16 = Pow2<u16>;
pub type Pow2u32 = Pow2<u32>;
pub type Pow2u64 = Pow2<u64>;
pub type Pow2u128 = Pow2<u128>;
pub type Pow2usize = Pow2<usize>;

impl_trait_signed_unsigned!(
    Div<UnboundedPow2>,
    signed_body {
        type Output = Self;

        #[inline(always)]
        fn div(self, other: UnboundedPow2) -> Self {
            debug_assert!(other.is_safe::<<Self as Int>::Unsigned>());
            let sgn = self >> (Self::BITS - 1);
            let msk = Self::mask(other.exponent as u32);
            (self + (sgn & msk)) >> other.exponent
        }
    },
    unsigned_body {
        type Output = Self;

        #[inline(always)]
        fn div(self, other: UnboundedPow2) -> Self {
            debug_assert!(other.is_safe::<<Self as Int>::Unsigned>());
            self >> other.exponent
        }
    }
);

impl_generic_trait_signed_unsigned!(
    <T> Div<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> },
    signed_body {
        type Output = Self;

        #[inline(always)]
        fn div(self, other: Pow2<T>) -> Self {
            // SAFETY: Pow2<T> guarantees a valid shift
            let sgn = unsafe { self.unchecked_shr(Self::BITS - 1) };
            // SAFETY: Pow2<T> guarantees a valid shift
            let msk = unsafe { Self::unchecked_mask(other.exponent as u32) };
            (self + (sgn & msk)) >> other.exponent
        }
    },
    unsigned_body {
        type Output = Self;

        #[inline(always)]
        #[allow(clippy::suspicious_arithmetic_impl)]
        fn div(self, other: Pow2<T>) -> Self {
            // SAFETY: Pow2<T> guarantees a valid shift
            unsafe { self.unchecked_shr(other.exponent as u32) }
        }
    }
);

impl_trait_all_ints!(
    DivAssign<UnboundedPow2> => {
        #[inline(always)]
        fn div_assign(&mut self, other: UnboundedPow2) {
            debug_assert!(other.is_safe::<<Self as Int>::Unsigned>());
            *self = *self / other;
        }
    }
);

impl_generic_trait_all_ints!(
    <T> DivAssign<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> } => {
        #[inline(always)]
        fn div_assign(&mut self, other: Pow2<T>) {
            *self = *self / other;
        }
    }
);
impl_trait_signed_unsigned!(
    Rem<UnboundedPow2>,
    signed_body {
        type Output = Self;

        #[inline(always)]
        fn rem(self, other: UnboundedPow2) -> Self {
            debug_assert!(other.is_safe::<<Self as Int>::Unsigned>());
            self - (self / other * other)
        }
    },
    unsigned_body {
        type Output = Self;

        #[inline(always)]
        fn rem(self, other: UnboundedPow2) -> Self {
            debug_assert!(other.is_safe::<<Self as Int>::Unsigned>());
            self - (self / other * other)
        }
    }
);

impl_generic_trait_signed_unsigned!(
    <T> Rem<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> },
    signed_body {
        type Output = Self;

        #[inline(always)]
        fn rem(self, other: Pow2<T>) -> Self {
            self - (self / other * other)
        }
    },
    unsigned_body {
        type Output = Self;

        #[inline(always)]
        fn rem(self, other: Pow2<T>) -> Self {
            self - (self / other * other)
        }
    }
);

impl_trait_all_ints!(
    RemAssign<UnboundedPow2> => {
        #[inline(always)]
        fn rem_assign(&mut self, other: UnboundedPow2) {
            *self = *self % other;
        }
    }
);

impl_generic_trait_all_ints!(
    <T> RemAssign<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> } => {
        #[inline(always)]
        fn rem_assign(&mut self, other: Pow2<T>) {
            *self = *self % other;
        }
    }
);

impl_trait_all_ints!(
    Mul<UnboundedPow2> => {
        type Output = Self;

        #[inline(always)]
        fn mul(self, other: UnboundedPow2) -> Self {
            debug_assert!(other.is_safe::<<Self as Int>::Unsigned>());
            // Check for overflow.
            debug_assert!(self == self << other.exponent >> other.exponent);
            self << other.exponent
        }
    }
);

impl_generic_trait_all_ints!(
    <T> Mul<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> } => {
        type Output = Self;

        #[inline(always)]
        fn mul(self, other: Pow2<T>) -> Self {
            // Check for overflow.
            debug_assert!(self == self << other.exponent >> other.exponent);
            // SAFETY: Pow2<T> guarantees a valid shift
            unsafe { self.unchecked_shl(other.exponent as u32) }
        }
    }
);

impl_trait_all_ints!(
    MulAssign<UnboundedPow2> => {
        #[inline(always)]
        fn mul_assign(&mut self, other: UnboundedPow2) {
            *self = *self * other;
        }
    }
);

impl_generic_trait_all_ints!(
    <T> MulAssign<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> } => {
        #[inline(always)]
        fn mul_assign(&mut self, other: Pow2<T>) {
            *self = *self * other;
        }
    }
);

macro_rules! make_func_trait {
    ($trait_name:ident, $func_name:ident) => {
        #[allow(rustdoc::broken_intra_doc_links)]
        #[doc = concat!(
            "Trait implementing `", stringify!($func_name), "` taking `self` and some `rhs` of type `Rhs`.\n\n",
            "Implementations of this trait define the behavior of the free ",
            "function [`", stringify!($func_name), "`] which dispatches to them.\n\n",
            "The [`crate::ipow2`] module may be implementing this trait for all primitive integer ",
            "types as `self` and the following `Rhs` types:\n",
            " - [`Pow2`] - [`docs?`](__detached_docs::Pow2::", stringify!($trait_name), ")\n",
            " - [`UnboundedPow2`] - [`docs?`](__detached_docs::UnboundedPow2::", stringify!($trait_name), ")\n",
        )]
        pub trait $trait_name<Rhs> {
            type Output;

            #[doc = concat!(
                "See docs for [`", stringify!($trait_name), "`]."
            )]
            #[must_use]
            #[allow(clippy::wrong_self_convention)]
            fn $func_name(self, rhs: Rhs) -> Self::Output;
        }

        #[doc = concat!(
            "Uses the [`", stringify!($trait_name), "`] trait for dispatch.\n\n",
            "Invokes [`", stringify!($trait_name), "::", stringify!($func_name), "`] ",
            "via `lhs.", stringify!($func_name), "(rhs)`.",
        )]
        #[must_use]
        #[inline(always)]
        pub fn $func_name<L, R>(lhs: L, rhs: R) -> L::Output
        where
            L: $trait_name<R>
        {
            lhs.$func_name(rhs)
        }
    };
}

make_func_trait!(CheckedMul, checked_mul);

/// See [docs](__detached_docs::UnboundedPow2::CheckedMul)
impl<T> CheckedMul<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedMul)
    #[inline(always)]
    fn checked_mul(self, rhs: UnboundedPow2) -> Self::Output {
        let result = self.checked_shl(rhs.exponent as u32)?;
        if self == result >> rhs.exponent {
            Some(result)
        } else {
            None
        }
    }
}

/// See [docs](__detached_docs::Pow2::CheckedMul)
impl<L, T> CheckedMul<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::Pow2::CheckedMul)
    #[inline(always)]
    fn checked_mul(self, rhs: Pow2<T>) -> Self::Output {
        // SAFETY: Pow2<T> guarantees a valid shift
        let result = unsafe { self.unchecked_shl(rhs.exponent as u32) };
        // SAFETY: Pow2<T> guarantees a valid shift
        if self == unsafe { result.unchecked_shr(rhs.exponent as u32) } {
            Some(result)
        } else {
            None
        }
    }
}

make_func_trait!(CheckedDiv, checked_div);

/// See [docs](__detached_docs::UnboundedPow2::CheckedDiv)
impl<T> CheckedDiv<UnboundedPow2> for T
where
    T: Int + Div<UnboundedPow2, Output = T>,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedDiv)
    #[inline(always)]
    fn checked_div(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>().then(|| self / rhs)
    }
}

make_func_trait!(UnboundedDiv, unbounded_div);

impl<T> UnboundedDiv<UnboundedPow2> for T
where
    T: Int + Div<UnboundedPow2, Output = T>,
{
    type Output = Self;

    #[inline(always)]
    fn unbounded_div(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            self / rhs
        } else {
            T::ZERO
        }
    }
}

make_func_trait!(CheckedRem, checked_rem);

/// See [docs](__detached_docs::UnboundedPow2::CheckedRem)
impl<T> CheckedRem<UnboundedPow2> for T
where
    T: Int + Rem<UnboundedPow2, Output = T>,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedRem)
    #[inline(always)]
    fn checked_rem(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>().then(|| self % rhs)
    }
}

make_func_trait!(UnboundedRem, unbounded_rem);

/// See [docs](__detached_docs::UnboundedPow2::CheckedRem)
impl<T> UnboundedRem<UnboundedPow2> for T
where
    T: Int + Rem<UnboundedPow2, Output = T>,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedRem)
    #[inline(always)]
    fn unbounded_rem(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            self % rhs
        } else {
            self
        }
    }
}

make_func_trait!(DivFloor, div_floor);

/// See [docs](__detached_docs::UnboundedPow2::DivFloor)
impl<T> DivFloor<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::DivFloor)
    #[inline(always)]
    fn div_floor(self, rhs: UnboundedPow2) -> Self::Output {
        debug_assert!(rhs.is_safe::<T::Unsigned>());
        self >> rhs.exponent
    }
}

/// See [docs](__detached_docs::Pow2::DivFloor)
impl<L, T> DivFloor<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Self;

    /// See [docs](__detached_docs::Pow2::DivFloor)
    #[inline(always)]
    fn div_floor(self, rhs: Pow2<T>) -> Self::Output {
        // SAFETY: Pow2<T> guarantees a valid shift
        unsafe { self.unchecked_shr(rhs.exponent as u32) }
    }
}

make_func_trait!(CheckedDivFloor, checked_div_floor);

/// See [docs](__detached_docs::UnboundedPow2::CheckedDivFloor)
impl<T> CheckedDivFloor<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedDivFloor)
    #[inline(always)]
    fn checked_div_floor(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>().then(|| div_floor(self, rhs))
    }
}

make_func_trait!(UnboundedDivFloor, unbounded_div_floor);

/// See [docs](__detached_docs::UnboundedPow2::UnboundedDivFloor)
impl<T> UnboundedDivFloor<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::UnboundedDivFloor)
    #[inline(always)]
    fn unbounded_div_floor(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            div_floor(self, rhs)
        } else if T::IS_UNSIGNED || self >= T::ZERO {
            T::ZERO
        } else {
            T::MINUS_ONE
        }
    }
}

make_func_trait!(RemFloor, rem_floor);

/// See [docs](__detached_docs::UnboundedPow2::RemFloor)
impl<T> RemFloor<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::RemFloor)
    #[inline(always)]
    fn rem_floor(self, rhs: UnboundedPow2) -> Self::Output {
        debug_assert!(rhs.is_safe::<T::Unsigned>());
        // The floor remainder of a power of two is exactly the low `exponent`
        // bits (`x & (2^e - 1)`), non-negative for signed inputs by construction.
        self & T::mask(rhs.exponent as u32)
    }
}

/// See [docs](__detached_docs::Pow2::RemFloor)
impl<L, T> RemFloor<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Self;

    /// See [docs](__detached_docs::Pow2::RemFloor)
    #[inline(always)]
    fn rem_floor(self, rhs: Pow2<T>) -> Self::Output {
        // The floor remainder of a power of two is exactly the low `exponent`
        // bits (`x & (2^e - 1)`), non-negative for signed inputs by construction.
        // SAFETY: Pow2<T> guarantees a valid shift
        self & unsafe { Self::unchecked_mask(rhs.exponent as u32) }
    }
}

make_func_trait!(CheckedRemFloor, checked_rem_floor);

/// See [docs](__detached_docs::UnboundedPow2::CheckedRemFloor)
impl<T> CheckedRemFloor<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedRemFloor)
    #[inline(always)]
    fn checked_rem_floor(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>().then(|| rem_floor(self, rhs))
    }
}

make_func_trait!(DivCeil, div_ceil);

/// See [docs](__detached_docs::UnboundedPow2::DivCeil)
impl<T> DivCeil<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::DivCeil)
    #[inline(always)]
    fn div_ceil(self, rhs: UnboundedPow2) -> Self::Output {
        debug_assert!(rhs.is_safe::<T::Unsigned>());
        // While slightly slower on a single value, this implementation is chosen
        // because it is better when performed with at least 2 values with the same rhs,
        // and also uses the same mask as other operations, and also vectorizable.
        let mask = T::mask(rhs.exponent as u32);
        let floored = div_floor(self, rhs);
        floored + T::from_bool((self & mask).is_not_zero())
    }
}

/// See [docs](__detached_docs::Pow2::DivCeil)
impl<L, T> DivCeil<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Self;

    /// See [docs](__detached_docs::Pow2::DivCeil)
    #[inline(always)]
    fn div_ceil(self, rhs: Pow2<T>) -> Self::Output {
        // SAFETY: Pow2<T> guarantees a valid shift
        let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
        let floored = div_floor(self, rhs);
        floored + Self::from_bool((self & mask).is_not_zero())
    }
}

make_func_trait!(CheckedDivCeil, checked_div_ceil);

/// See [docs](__detached_docs::UnboundedPow2::CheckedDivCeil)
impl<T> CheckedDivCeil<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedDivCeil)
    #[inline(always)]
    fn checked_div_ceil(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>().then(|| div_ceil(self, rhs))
    }
}

make_func_trait!(UnboundedDivCeil, unbounded_div_ceil);

/// See [docs](__detached_docs::UnboundedPow2::UnboundedDivCeil)
impl<T> UnboundedDivCeil<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::UnboundedDivCeil)
    #[inline(always)]
    fn unbounded_div_ceil(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            div_ceil(self, rhs)
        } else if self <= T::ZERO {
            T::ZERO
        } else {
            T::ONE
        }
    }
}

make_func_trait!(DivRound, div_round);

impl_trait_signed_unsigned!(
    DivRound<UnboundedPow2>,
    signed_body {
        //! [docs](__detached_docs::UnboundedPow2::DivRound)
        type Output = Self;

        /// See [docs](__detached_docs::UnboundedPow2::DivRound)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn div_round(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_SIGNED);
            debug_assert!(rhs.is_safe::<<Self as Int>::Unsigned>());
            let mask = Self::mask(rhs.exponent as u32);
            let floored = div_floor(self, rhs);
            // Account for signedness of a
            // for a >= 0 we need rem != 0 and rem >= mask_highest_bit
            // for a < 0 we need rem > mask_highest_bit
            // make sure the saturating sub saturates to zero (unsigned)
            let rem = (self & mask)
                .cast_unsigned()
                .saturating_sub(<Self as Int>::Unsigned::from_bool(self < Self::ZERO));
            let rems = (rem << 1) >> rhs.exponent;
            floored + Self::self_from_unsigned(rems & <Self as Int>::Unsigned::ONE)
        }
    },
    unsigned_body {
        //! [docs](__detached_docs::UnboundedPow2::DivRound)
        type Output = Self;

        /// See [docs](__detached_docs::UnboundedPow2::DivRound)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn div_round(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_UNSIGNED);
            debug_assert!(rhs.is_safe::<<Self as Int>::Unsigned>());
            let bit = Self::highest_mask_bit(rhs.exponent as u32);
            let floored = div_floor(self, rhs);
            // this is only valid for unsigned a
            floored + Self::from_bool((self & bit).is_not_zero())
        }
    }
);

impl_generic_trait_signed_unsigned!(
    <T> DivRound<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> },
    signed_body {
        //! See [docs](__detached_docs::Pow2::DivRound)
        type Output = Self;

        /// See [docs](__detached_docs::Pow2::DivRound)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn div_round(self, rhs: Pow2<T>) -> Self {
            debug_assert!(Self::IS_SIGNED);
            // SAFETY: Pow2<T> guarantees a valid shift
            let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
            let floored = div_floor(self, rhs);
            // Account for signedness of a
            // for a >= 0 we need rem != 0 and rem >= mask_highest_bit
            // for a < 0 we need rem > mask_highest_bit
            // make sure the saturating sub saturates to zero (unsigned)
            let rem = (self & mask)
                .cast_unsigned()
                .saturating_sub(<Self as Int>::Unsigned::from_bool(self < Self::ZERO));
            // SAFETY: Pow2<T> guarantees a valid shift
            let rems = unsafe { rem.unchecked_shl(1).unchecked_shr(rhs.exponent as u32) };
            floored + Self::self_from_unsigned(rems & <Self as Int>::Unsigned::ONE)
        }
    },
    unsigned_body {
        //! See [docs](__detached_docs::Pow2::DivRound)
        type Output = Self;

        /// See [docs](__detached_docs::Pow2::DivRound)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn div_round(self, rhs: Pow2<T>) -> Self {
            debug_assert!(Self::IS_UNSIGNED);
            // SAFETY: Pow2<T> guarantees a valid shift
            let bit = unsafe { Self::unchecked_highest_mask_bit(rhs.exponent as u32) };
            let floored = div_floor(self, rhs);
            // this is only valid for unsigned a
            floored + Self::from_bool((self & bit).is_not_zero())
        }
    }
);

make_func_trait!(CheckedDivRound, checked_div_round);

/// See [docs](__detached_docs::UnboundedPow2::CheckedDivRound)
impl<T> CheckedDivRound<UnboundedPow2> for T
where
    T: Int + DivRound<UnboundedPow2, Output = Self>,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedDivRound)
    #[inline(always)]
    fn checked_div_round(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>().then(|| div_round(self, rhs))
    }
}

make_func_trait!(UnboundedDivRound, unbounded_div_round);

/// See [docs](__detached_docs::UnboundedPow2::UnboundedDivRound)
impl<T> UnboundedDivRound<UnboundedPow2> for T
where
    T: Int + DivRound<UnboundedPow2, Output = Self>,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::UnboundedDivRound)
    #[inline(always)]
    fn unbounded_div_round(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            div_round(self, rhs)
        } else if T::IS_SIGNED && self == T::MIN && rhs.exponent as u32 == T::BITS {
            // result would be -0.5, so round to -1
            T::MINUS_ONE
        } else if T::IS_UNSIGNED && self >= (T::MAX >> 1) + T::ONE && rhs.exponent as u32 == T::BITS
        {
            // result would be >= 0.5, so round away from zero to 1
            T::ONE
        } else {
            T::ZERO
        }
    }
}

make_func_trait!(IsMultipleOf, is_multiple_of);

/// See [docs](__detached_docs::UnboundedPow2::IsMultipleOf)
impl<T> IsMultipleOf<UnboundedPow2> for T
where
    T: Int,
{
    type Output = bool;

    /// See [docs](__detached_docs::UnboundedPow2::IsMultipleOf)
    #[inline(always)]
    fn is_multiple_of(self, rhs: UnboundedPow2) -> Self::Output {
        debug_assert!(rhs.is_safe::<T::Unsigned>());
        // `x` is a multiple of `2^e` iff its low `e` bits are clear. Mask-based
        // (AND + test) rather than `trailing_zeros`, which does not vectorize.
        (self & T::mask(rhs.exponent as u32)).is_zero()
    }
}

/// See [docs](__detached_docs::Pow2::IsMultipleOf)
impl<L, T> IsMultipleOf<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = bool;

    /// See [docs](__detached_docs::Pow2::IsMultipleOf)
    #[inline(always)]
    fn is_multiple_of(self, rhs: Pow2<T>) -> Self::Output {
        // `x` is a multiple of `2^e` iff its low `e` bits are clear. Mask-based
        // (AND + test) rather than `trailing_zeros`, which does not vectorize.
        // SAFETY: Pow2<T> guarantees a valid shift
        (self & unsafe { Self::unchecked_mask(rhs.exponent as u32) }).is_zero()
    }
}

make_func_trait!(UnboundedIsMultipleOf, unbounded_is_multiple_of);

/// See [docs](__detached_docs::UnboundedPow2::UnboundedIsMultipleOf)
impl<T> UnboundedIsMultipleOf<UnboundedPow2> for T
where
    T: Int,
{
    type Output = bool;

    /// See [docs](__detached_docs::UnboundedPow2::UnboundedIsMultipleOf)
    #[inline(always)]
    fn unbounded_is_multiple_of(self, rhs: UnboundedPow2) -> Self::Output {
        self.is_zero() || self.trailing_zeros() >= rhs.exponent as u32
    }
}

make_func_trait!(FloorToMultiple, floor_to_multiple);

/// See [docs](__detached_docs::UnboundedPow2::FloorToMultiple)
impl<T> FloorToMultiple<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::FloorToMultiple)
    #[inline(always)]
    fn floor_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
        debug_assert!(rhs.is_safe::<T::Unsigned>());
        // Clear the low `exponent` bits (`x & !(2^e - 1)`). Mask-based rather
        // than `>> e << e` so the mask is shared with the other operations and
        // the code vectorizes (no variable shift).
        self & !T::mask(rhs.exponent as u32)
    }
}

/// See [docs](__detached_docs::Pow2::FloorToMultiple)
impl<L, T> FloorToMultiple<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Self;

    /// See [docs](__detached_docs::Pow2::FloorToMultiple)
    #[inline(always)]
    fn floor_to_multiple(self, rhs: Pow2<T>) -> Self::Output {
        // Clear the low `exponent` bits (`x & !(2^e - 1)`). Mask-based rather
        // than `>> e << e` so the mask is shared with the other operations and
        // the code vectorizes (no variable shift).
        // SAFETY: Pow2<T> guarantees a valid shift
        self & !unsafe { Self::unchecked_mask(rhs.exponent as u32) }
    }
}

make_func_trait!(CheckedFloorToMultiple, checked_floor_to_multiple);

/// See [docs](__detached_docs::UnboundedPow2::CheckedFloorToMultiple)
impl<T> CheckedFloorToMultiple<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedFloorToMultiple)
    #[inline(always)]
    fn checked_floor_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
        rhs.is_safe::<T::Unsigned>()
            .then(|| floor_to_multiple(self, rhs))
    }
}

make_func_trait!(UnboundedFloorToMultiple, unbounded_floor_to_multiple);

impl_trait_signed_unsigned!(
    UnboundedFloorToMultiple<UnboundedPow2>,
    signed_body {
        //! See [docs](__detached_docs::UnboundedPow2::UnboundedFloorToMultiple)
        type Output = Option<Self>;

        /// See [docs](__detached_docs::UnboundedPow2::UnboundedFloorToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn unbounded_floor_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_SIGNED);
            if rhs.is_safe::<<Self as Int>::Unsigned>() {
                Some(floor_to_multiple(self, rhs))
            } else if self >= Self::ZERO {
                Some(Self::ZERO)
            } else {
                None
            }
        }
    },
    unsigned_body {
        //! See [docs](__detached_docs::UnboundedPow2::UnboundedFloorToMultiple)
        type Output = Self;

        /// See [docs](__detached_docs::UnboundedPow2::UnboundedFloorToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn unbounded_floor_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_UNSIGNED);
            if rhs.is_safe::<Self>() {
                floor_to_multiple(self, rhs)
            } else {
                Self::ZERO
            }
        }
    }
);

make_func_trait!(CeilToMultiple, ceil_to_multiple);

/// See [docs](__detached_docs::UnboundedPow2::CeilToMultiple)
impl<T> CeilToMultiple<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Self;

    /// See [docs](__detached_docs::UnboundedPow2::CeilToMultiple)
    #[inline(always)]
    fn ceil_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
        debug_assert!(rhs.is_safe::<T::Unsigned>());
        // We can actually use the mask method here because if the intermediate `a + mask` overflows
        // then the actual result would overflow too.
        let mask = T::mask(rhs.exponent as u32);
        (self + mask) & !mask
    }
}

/// See [docs](__detached_docs::Pow2::CeilToMultiple)
impl<L, T> CeilToMultiple<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Self;

    /// See [docs](__detached_docs::Pow2::CeilToMultiple)
    #[inline(always)]
    fn ceil_to_multiple(self, rhs: Pow2<T>) -> Self::Output {
        // We can actually use the mask method here because if the intermediate `a + mask` overflows
        // then the actual result would overflow too.
        // SAFETY: Pow2<T> guarantees a valid shift
        let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
        (self + mask) & !mask
    }
}

make_func_trait!(CheckedCeilToMultiple, checked_ceil_to_multiple);

/// See [docs](__detached_docs::UnboundedPow2::CheckedCeilToMultiple)
impl<T> CheckedCeilToMultiple<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::CheckedCeilToMultiple)
    #[inline(always)]
    fn checked_ceil_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            let mask = T::mask(rhs.exponent as u32);
            Some(self.checked_add(mask)? & !mask)
        } else {
            None
        }
    }
}

/// See [docs](__detached_docs::Pow2::CheckedCeilToMultiple)
impl<L, T> CheckedCeilToMultiple<Pow2<T>> for L
where
    L: IntAtLeastAsWide<T>,
    T: UnsignedInt,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::Pow2::CheckedCeilToMultiple)
    #[inline(always)]
    fn checked_ceil_to_multiple(self, rhs: Pow2<T>) -> Self::Output {
        // SAFETY: Pow2<T> guarantees a valid shift
        let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
        Some(self.checked_add(mask)? & !mask)
    }
}

make_func_trait!(UnboundedCeilToMultiple, unbounded_ceil_to_multiple);

/// See [docs](__detached_docs::UnboundedPow2::UnboundedCeilToMultiple)
impl<T> UnboundedCeilToMultiple<UnboundedPow2> for T
where
    T: Int,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::UnboundedCeilToMultiple)
    #[inline(always)]
    fn unbounded_ceil_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            checked_ceil_to_multiple(self, rhs)
        } else if self <= T::ZERO {
            Some(T::ZERO)
        } else {
            None
        }
    }
}

make_func_trait!(RoundToMultiple, round_to_multiple);

impl_trait_signed_unsigned!(
    RoundToMultiple<UnboundedPow2>,
    signed_body {
        //! See [docs](__detached_docs::UnboundedPow2::RoundToMultiple)
        type Output = Self;

        /// See [docs](__detached_docs::UnboundedPow2::RoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn round_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_SIGNED);
            debug_assert!(rhs.is_safe::<<Self as Int>::Unsigned>());
            let mask = Self::mask(rhs.exponent as u32);
            let bit = Self::self_from_unsigned(
                // Bias:
                <Self as Int>::Unsigned::highest_mask_bit(rhs.exponent as u32)
                // We bias the bias for negative lhs to get correct rounding away from zero.
                // Saturating sub must be against zero (so unsigned)
                .saturating_sub(
                    <Self as Int>::Unsigned::from_bool(self < Self::ZERO)
                )
            );

            // We can actually use the mask method here because if the intermediate `a + mask` overflows
            // then the actual result would overflow too.
            (self + bit) & !mask
        }
    },
    unsigned_body {
        //! See [docs](__detached_docs::UnboundedPow2::RoundToMultiple)
        type Output = Self;

        /// See [docs](__detached_docs::UnboundedPow2::RoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn round_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_UNSIGNED);
            debug_assert!(rhs.is_safe::<<Self as Int>::Unsigned>());
            let mask = Self::mask(rhs.exponent as u32);
            let bit = Self::highest_mask_bit(rhs.exponent as u32);
            // We can actually use the mask method here because if the intermediate `a + mask` overflows
            // then the actual result would overflow too.
            (self + bit) & !mask
        }
    }
);

impl_generic_trait_signed_unsigned!(
    <T> RoundToMultiple<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> },
    signed_body {
        //! See [docs](__detached_docs::Pow2::RoundToMultiple)
        type Output = Self;

        /// See [docs](__detached_docs::Pow2::RoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn round_to_multiple(self, rhs: Pow2<T>) -> Self {
            debug_assert!(Self::IS_SIGNED);
            // SAFETY: Pow2<T> guarantees a valid shift
            let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
            let bit = Self::self_from_unsigned(
                // Bias:
                // SAFETY: Pow2<T> guarantees a valid shift
                unsafe { <Self as Int>::Unsigned::unchecked_highest_mask_bit(rhs.exponent as u32) }
                // We bias the bias for negative lhs to get correct rounding away from zero.
                // Saturating sub must be against zero (so unsigned)
                .saturating_sub(
                    <Self as Int>::Unsigned::from_bool(self < Self::ZERO)
                )
            );

            // We can actually use the mask method here because if the intermediate `a + mask` overflows
            // then the actual result would overflow too.
            (self + bit) & !mask
        }
    },
    unsigned_body {
        //! See [docs](__detached_docs::Pow2::RoundToMultiple)
        type Output = Self;

        /// See [docs](__detached_docs::Pow2::RoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn round_to_multiple(self, rhs: Pow2<T>) -> Self::Output {
            debug_assert!(Self::IS_UNSIGNED);
            // SAFETY: Pow2<T> guarantees a valid shift
            let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
            // SAFETY: Pow2<T> guarantees a valid shift
            let bit = unsafe { Self::unchecked_highest_mask_bit(rhs.exponent as u32) };
            // We can actually use the mask method here because if the intermediate `a + mask` overflows
            // then the actual result would overflow too.
            (self + bit) & !mask
        }
    }
);

make_func_trait!(CheckedRoundToMultiple, checked_round_to_multiple);

impl_trait_signed_unsigned!(
    CheckedRoundToMultiple<UnboundedPow2>,
    signed_body {
        //! See [docs](__detached_docs::UnboundedPow2::CheckedRoundToMultiple)
        type Output = Option<Self>;

        /// See [docs](__detached_docs::UnboundedPow2::CheckedRoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn checked_round_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_SIGNED);
            if !rhs.is_safe::<<Self as Int>::Unsigned>() {
                return None;
            }

            let mask = Self::mask(rhs.exponent as u32);
            let bit = Self::self_from_unsigned(
                // Bias:
                <Self as Int>::Unsigned::highest_mask_bit(rhs.exponent as u32)
                // We bias the bias for negative lhs to get correct rounding away from zero.
                // Saturating sub must be against zero (so unsigned)
                .saturating_sub(
                    <Self as Int>::Unsigned::from_bool(self < Self::ZERO)
                )
            );

            Some(self.checked_add(bit)? & !mask)
        }
    },
    unsigned_body {
        //! See [docs](__detached_docs::UnboundedPow2::CheckedRoundToMultiple)
        type Output = Option<Self>;

        /// See [docs](__detached_docs::UnboundedPow2::CheckedRoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn checked_round_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
            debug_assert!(Self::IS_UNSIGNED);
            if !rhs.is_safe::<<Self as Int>::Unsigned>() {
                return None;
            }

            let mask = Self::mask(rhs.exponent as u32);
            let bit = Self::highest_mask_bit(rhs.exponent as u32);

            Some(self.checked_add(bit)? & !mask)
        }
    }
);

impl_generic_trait_signed_unsigned!(
    <T> CheckedRoundToMultiple<Pow2<T>> where { T: UnsignedInt, Self: IntAtLeastAsWide<T> },
    signed_body {
        //! See [docs](__detached_docs::Pow2::CheckedRoundToMultiple)
        type Output = Option<Self>;

        /// See [docs](__detached_docs::Pow2::CheckedRoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn checked_round_to_multiple(self, rhs: Pow2<T>) -> Self::Output {
            debug_assert!(Self::IS_SIGNED);
            // SAFETY: Pow2<T> guarantees a valid shift
            let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
            let bit = Self::self_from_unsigned(
                // Bias:
                // SAFETY: Pow2<T> guarantees a valid shift
                unsafe { <Self as Int>::Unsigned::unchecked_highest_mask_bit(rhs.exponent as u32) }
                // We bias the bias for negative lhs to get correct rounding away from zero.
                // Saturating sub must be against zero (so unsigned)
                .saturating_sub(
                    <Self as Int>::Unsigned::from_bool(self < Self::ZERO)
                )
            );

            Some(self.checked_add(bit)? & !mask)
        }
    },
    unsigned_body {
        //! See [docs](__detached_docs::Pow2::CheckedRoundToMultiple)
        type Output = Option<Self>;

        /// See [docs](__detached_docs::Pow2::CheckedRoundToMultiple)
        #[inline(always)]
        #[allow(clippy::assertions_on_constants)]
        fn checked_round_to_multiple(self, rhs: Pow2<T>) -> Self::Output {
            debug_assert!(Self::IS_UNSIGNED);
            // SAFETY: Pow2<T> guarantees a valid shift
            let mask = unsafe { Self::unchecked_mask(rhs.exponent as u32) };
            // SAFETY: Pow2<T> guarantees a valid shift
            let bit = unsafe { Self::unchecked_highest_mask_bit(rhs.exponent as u32) };

            Some(self.checked_add(bit)? & !mask)
        }
    }
);

make_func_trait!(UnboundedRoundToMultiple, unbounded_round_to_multiple);

/// See [docs](__detached_docs::UnboundedPow2::UnboundedRoundToMultiple)
impl<T> UnboundedRoundToMultiple<UnboundedPow2> for T
where
    T: Int + CheckedRoundToMultiple<UnboundedPow2, Output = Option<Self>>,
{
    type Output = Option<Self>;

    /// See [docs](__detached_docs::UnboundedPow2::UnboundedRoundToMultiple)
    #[inline(always)]
    fn unbounded_round_to_multiple(self, rhs: UnboundedPow2) -> Self::Output {
        if rhs.is_safe::<T::Unsigned>() {
            checked_round_to_multiple(self, rhs)
        } else if Self::IS_SIGNED && self == Self::MIN && rhs.exponent as u32 == Self::BITS {
            // would round to twice Self::MIN
            None
        } else if Self::IS_UNSIGNED
            && self >= ((Self::MAX >> 1) + Self::ONE)
            && rhs.exponent as u32 == Self::BITS
        {
            // would round to Self::MAX + 1
            None
        } else {
            Some(T::ZERO)
        }
    }
}

#[allow(nonstandard_style)]
pub mod __detached_docs {
    macro_rules! colored_link {
        ($name:ident, $color:literal) => {
            concat!(
                "[<font color=\"",
                $color,
                "\">",
                stringify!($name),
                "</font>]",
                "(crate::",
                stringify!($name),
                ")"
            )
        };
    }

    macro_rules! trait_link {
        ($trait_name:ident) => {
            colored_link!($trait_name, "#b78cf2")
        };
    }

    macro_rules! struct_link {
        ($struct_name:ident) => {
            colored_link!($struct_name, "#2dbfb8")
        };
    }

    macro_rules! fn_link {
        ($fn_name:ident) => {
            colored_link!($fn_name, "#2bab63")
        };
    }

    #[rustfmt::skip]
    macro_rules! trait_code_header_pow2 {
        ($trait_name:ident, $func_name:ident, [$($additional_trait_bound:ident),*]) => {
            concat!(
                "<span><pre>",
                "impl&lt;L, T&gt; ", trait_link!($trait_name), "&lt;", struct_link!(Pow2), "&lt;T&gt;&gt; for L<br/>",
                "where<br/>",
                "&nbsp;&nbsp;&nbsp;&nbsp;L: ", trait_link!(IntAtLeastAsWide), "&lt;T&gt;", $(" + ", trait_link!($additional_trait_bound), "&lt;", struct_link!(Pow2), "&lt;T&gt;, Output = T&gt;",)* ",<br/>",
                "&nbsp;&nbsp;&nbsp;&nbsp;T: ", trait_link!(UnsignedInt), "<br/>",
                "fn ", fn_link!($func_name), "(self, rhs: ", struct_link!(Pow2), "&lt;T&gt;) -> Self::Output",
                "</pre></span><hr/>"
            )
        };
    }

    #[rustfmt::skip]
    macro_rules! trait_code_header_unb_pow2 {
        ($trait_name:ident, $func_name:ident, [$($additional_trait_bound:ident),*]) => {
            concat!(
                "<span><pre>",
                "impl&lt;T&gt; ", trait_link!($trait_name), "&lt;", struct_link!(UnboundedPow2), "&gt; for T<br/>",
                "where<br/>",
                "&nbsp;&nbsp;&nbsp;&nbsp;T: ", trait_link!(Int), $(" + ", trait_link!($additional_trait_bound), "&lt;", struct_link!(UnboundedPow2), ", Output = T&gt;",)* ",<br/>",
                "fn ", fn_link!($func_name), "(self, rhs: ", struct_link!(UnboundedPow2), ") -> Self::Output",
                "</pre></span><hr/>"
            )
        };
    }

    pub mod Pow2 {
        #[doc = trait_code_header_pow2!(CheckedMul, checked_mul, [])]
        /// Attempts to multiply `self` by `rhs`.
        /// Avoids integer multiplication by using bitwise arithmetic.
        ///
        /// Returns `None` if the result overflows, `Some(result)` otherwise.
        pub mod CheckedMul {}

        #[doc = trait_code_header_pow2!(DivFloor, div_floor, [])]
        /// Divides `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod DivFloor {}

        #[doc = trait_code_header_pow2!(RemFloor, rem_floor, [])]
        /// Computes the remainder of dividing `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod RemFloor {}

        #[doc = trait_code_header_pow2!(DivCeil, div_ceil, [])]
        /// Divides `self` by `rhs` with rounding towards positive infinity.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod DivCeil {}

        #[doc = trait_code_header_pow2!(DivRound, div_round, [])]
        /// Divides `self` by `rhs` with rounding to the nearest integer, ties resolving away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod DivRound {}

        #[doc = trait_code_header_pow2!(IsMultipleOf, is_multiple_of, [])]
        /// Returns whether `self` is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod IsMultipleOf {}

        #[doc = trait_code_header_pow2!(FloorToMultiple, floor_to_multiple, [])]
        /// Returns the largest integer `<=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod FloorToMultiple {}

        #[doc = trait_code_header_pow2!(CeilToMultiple, ceil_to_multiple, [])]
        /// Returns the smallest integer `>=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if the result is not representable.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod CeilToMultiple {}

        #[doc = trait_code_header_pow2!(CheckedCeilToMultiple, checked_ceil_to_multiple, [])]
        /// Returns the smallest integer `>=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable, `Some(result)` otherwise.
        pub mod CheckedCeilToMultiple {}

        #[doc = trait_code_header_pow2!(RoundToMultiple, round_to_multiple, [])]
        /// Returns the nearest integer to `self` that is a multiple of `rhs`, ties resolved away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if the result is not representable.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod RoundToMultiple {}

        #[doc = trait_code_header_pow2!(CheckedRoundToMultiple, checked_round_to_multiple, [])]
        /// Returns the nearest integer to `self` that is a multiple of `rhs`, ties resolved away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable, `Some(result)` otherwise.
        pub mod CheckedRoundToMultiple {}
    }

    pub mod UnboundedPow2 {
        #[doc = trait_code_header_unb_pow2!(CheckedMul, checked_mul, [])]
        /// Attempts to multiply `self` by `rhs`.
        /// Avoids integer multiplication by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable or the `rhs` exponent is too large,
        /// `Some(result)` otherwise.
        pub mod CheckedMul {}

        #[doc = trait_code_header_unb_pow2!(CheckedDiv, checked_div, [Div])]
        /// Attempts to divide `self` by `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedDiv {}

        #[doc = trait_code_header_unb_pow2!(UnboundedDiv, unbounded_div, [Div])]
        /// Divides `self` by `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod UnboundedDiv {}

        #[doc = trait_code_header_unb_pow2!(CheckedRem, checked_rem, [Rem])]
        /// Attempts to divide `self` by `rhs` and returns the remainer.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedRem {}

        #[doc = trait_code_header_unb_pow2!(CheckedRem, checked_rem, [Rem])]
        /// Divide `self` by `rhs` and returns the remainer.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod UnboundedRem {}

        #[doc = trait_code_header_unb_pow2!(DivFloor, div_floor, [])]
        /// Divides `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod DivFloor {}

        #[doc = trait_code_header_unb_pow2!(CheckedDivFloor, checked_div_floor, [])]
        /// Attempts to divide `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedDivFloor {}

        #[doc = trait_code_header_unb_pow2!(UnboundedDivFloor, unbounded_div_floor, [])]
        /// Divides `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod UnboundedDivFloor {}

        #[doc = trait_code_header_unb_pow2!(RemFloor, rem_floor, [])]
        /// Computes the remainder of dividing `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod RemFloor {}

        #[doc = trait_code_header_unb_pow2!(CheckedRemFloor, checked_rem_floor, [])]
        /// Attempts to compute the remainder of dividing `self` by `rhs` with rounding towards negative infinity.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedRemFloor {}

        #[doc = trait_code_header_unb_pow2!(DivCeil, div_ceil, [])]
        /// Divides `self` by `rhs` with rounding towards positive infinity.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod DivCeil {}

        #[doc = trait_code_header_unb_pow2!(CheckedDivCeil, checked_div_ceil, [])]
        /// Attempts to divide `self` by `rhs` with rounding towards positive infinity.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedDivCeil {}

        #[doc = trait_code_header_unb_pow2!(UnboundedDivCeil, unbounded_div_ceil, [])]
        /// Divides `self` by `rhs` with rounding towards positive infinity.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod UnboundedDivCeil {}

        #[doc = trait_code_header_unb_pow2!(DivRound, div_round, [])]
        /// Divides `self` by `rhs` with rounding to the nearest integer, ties resolving away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod DivRound {}

        #[doc = trait_code_header_unb_pow2!(CheckedDivRound, checked_div_round, [DivRound])]
        /// Attempts to divide `self` by `rhs` with rounding to the nearest integer, ties resolving away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedDivRound {}

        #[doc = trait_code_header_unb_pow2!(UnboundedDivRound, unbounded_div_round, [DivRound])]
        /// Divides `self` by `rhs` with rounding to the nearest integer, ties resolving away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod UnboundedDivRound {}

        #[doc = trait_code_header_unb_pow2!(IsMultipleOf, is_multiple_of, [])]
        /// Returns whether `self` is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod IsMultipleOf {}

        #[doc = trait_code_header_unb_pow2!(UnboundedIsMultipleOf, unbounded_is_multiple_of, [])]
        /// Returns whether `self` is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        pub mod UnboundedIsMultipleOf {}

        #[doc = trait_code_header_unb_pow2!(FloorToMultiple, floor_to_multiple, [])]
        /// Returns the largest integer `<=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod FloorToMultiple {}

        #[doc = trait_code_header_unb_pow2!(CheckedFloorToMultiple, checked_floor_to_multiple, [])]
        /// Returns the largest integer `<=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedFloorToMultiple {}

        #[doc = trait_code_header_unb_pow2!(UnboundedFloorToMultiple, unbounded_floor_to_multiple, [])]
        /// Returns the largest integer `<=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable, `Some(result)` otherwise.
        pub mod UnboundedFloorToMultiple {}

        #[doc = trait_code_header_unb_pow2!(CeilToMultiple, ceil_to_multiple, [])]
        /// Returns the smallest integer `>=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod CeilToMultiple {}

        #[doc = trait_code_header_unb_pow2!(CheckedCeilToMultiple, checked_ceil_to_multiple, [])]
        /// Returns the smallest integer `>=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable or the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedCeilToMultiple {}

        #[doc = trait_code_header_unb_pow2!(UnboundedCeilToMultiple, unbounded_ceil_to_multiple, [])]
        /// Returns the smallest integer `>=self` that is a multiple of `rhs`.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable, `Some(result)` otherwise.
        pub mod UnboundedCeilToMultiple {}

        #[doc = trait_code_header_unb_pow2!(RoundToMultiple, round_to_multiple, [])]
        /// Returns the nearest integer to `self` that is a multiple of `rhs`, ties resolved away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// # Panics
        ///
        /// In debug builds, panics if `rhs` is not safely representable as `T::Unsigned`.
        /// In release builds this check is skipped; the result will be defined but incorrect.
        pub mod RoundToMultiple {}

        #[doc = trait_code_header_unb_pow2!(CheckedRoundToMultiple, checked_round_to_multiple, [])]
        /// Returns the nearest integer to `self` that is a multiple of `rhs`, ties resolved away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable or the `rhs` exponent is too large, `Some(result)` otherwise.
        pub mod CheckedRoundToMultiple {}

        #[doc = trait_code_header_unb_pow2!(UnboundedRoundToMultiple, unbounded_round_to_multiple, [])]
        /// Returns the nearest integer to `self` that is a multiple of `rhs`, ties resolved away from zero.
        /// Avoids integer division by using bitwise arithmetic.
        ///
        /// Returns `None` if the result is not representable, `Some(result)` otherwise.
        pub mod UnboundedRoundToMultiple {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;
    use std::hash::{DefaultHasher, Hash, Hasher};

    #[test]
    fn unb_pow2_constructible_from_any_u8_exponent() {
        for e in 0_u8..=u8::MAX {
            assert_eq!(UnboundedPow2::from_exponent(e).exponent(), e);
        }
    }

    #[test]
    fn unb_pow2_constructible_from_align_of() {
        assert_eq!(
            UnboundedPow2::align_of::<()>(),
            UnboundedPow2::from_exponent(0)
        );
        assert_eq!(
            UnboundedPow2::align_of::<u8>(),
            UnboundedPow2::from_exponent(0)
        );
        assert_eq!(
            UnboundedPow2::align_of::<u32>(),
            UnboundedPow2::from_exponent(2)
        );
        assert_eq!(
            UnboundedPow2::align_of::<u128>(),
            UnboundedPow2::from_exponent(4)
        );
    }

    #[test]
    fn unb_pow2_constructible_from_align_of_val() {
        assert_eq!(
            UnboundedPow2::align_of_val(&()),
            UnboundedPow2::from_exponent(0)
        );
        assert_eq!(
            UnboundedPow2::align_of_val(&0_u8),
            UnboundedPow2::from_exponent(0)
        );
        assert_eq!(
            UnboundedPow2::align_of_val(&0_u32),
            UnboundedPow2::from_exponent(2)
        );
        assert_eq!(
            UnboundedPow2::align_of_val(&0_u128),
            UnboundedPow2::from_exponent(4)
        );
    }

    #[test]
    fn unb_pow2_try_from_power_of_two_int() {
        assert_eq!(
            UnboundedPow2::try_from(32_i8),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_u8),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_i16),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_u16),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_i32),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_u32),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_i64),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_u64),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_i128),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_u128),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_isize),
            Ok(UnboundedPow2::from_exponent(5))
        );
        assert_eq!(
            UnboundedPow2::try_from(32_usize),
            Ok(UnboundedPow2::from_exponent(5))
        );
    }

    #[test]
    fn pow2_try_from_power_of_two_int() {
        assert_eq!(
            Pow2::<u8>::try_from(32_i8),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_u8),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_i16),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_u16),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_i32),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_u32),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_i64),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_u64),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_i128),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_u128),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_isize),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(32_usize),
            Ok(Pow2::<u8>::from_exponent(5).unwrap())
        );
    }

    #[test]
    fn unb_pow2_try_from_near_power_of_two() {
        assert_eq!(UnboundedPow2::try_from(31_i8), Err(NotPow2));
        assert_eq!(UnboundedPow2::try_from(33_i8), Err(NotPow2));
    }

    #[test]
    fn pow2_try_from_near_power_of_two() {
        assert_eq!(
            Pow2::<u8>::try_from(31_i8),
            Err(Pow2TryFromIntError::NotPow2)
        );
        assert_eq!(
            Pow2::<u8>::try_from(33_i8),
            Err(Pow2TryFromIntError::NotPow2)
        );
    }

    #[test]
    fn pow2_try_from_out_of_range() {
        assert_eq!(
            Pow2::<u8>::try_from(i32::MAX as u32 + 1),
            Err(Pow2TryFromIntError::Pow2OutOfRange)
        );
        assert_eq!(
            Pow2::<u8>::try_from(i16::MAX as u64 + 1),
            Err(Pow2TryFromIntError::Pow2OutOfRange)
        );
    }

    #[test]
    fn unb_pow2_try_from_zero() {
        assert_eq!(UnboundedPow2::try_from(0_i8), Err(NotPow2));
    }

    #[test]
    fn unb_pow2_try_from_negative() {
        assert_eq!(UnboundedPow2::try_from(-1_i8), Err(NotPow2));
    }

    #[test]
    fn unb_pow2_try_from_min() {
        assert_eq!(UnboundedPow2::try_from(i64::MIN), Err(NotPow2));
    }

    #[test]
    fn unb_pow2_try_from_max() {
        assert_eq!(UnboundedPow2::try_from(i64::MAX), Err(NotPow2));
    }

    #[test]
    fn unb_pow2_from_pow2() {
        assert_eq!(
            UnboundedPow2::from(Pow2::<u32>::from_exponent(13).unwrap()).exponent(),
            13
        );
        assert_eq!(
            UnboundedPow2::from(Pow2::<u128>::from_exponent(127).unwrap()).exponent(),
            127
        );
    }

    #[test]
    fn pow2_try_from_pow2() {
        assert_eq!(
            Pow2::<u32>::try_from(UnboundedPow2::from_exponent(123)),
            Err(Pow2OutOfRange)
        );
        assert_eq!(
            Pow2::<u64>::try_from(UnboundedPow2::from_exponent(123)),
            Err(Pow2OutOfRange)
        );
        assert_eq!(
            Pow2::<u128>::try_from(UnboundedPow2::from_exponent(123)),
            Ok(Pow2::<u128>::from_exponent(123).unwrap())
        );
    }

    #[test]
    fn unb_pow2_as_int_all_types() {
        let p = UnboundedPow2::from_exponent(6);
        assert_eq!(p.as_i8(), 64);
        assert_eq!(p.as_u8(), 64);
        assert_eq!(p.as_i16(), 64);
        assert_eq!(p.as_u16(), 64);
        assert_eq!(p.as_i32(), 64);
        assert_eq!(p.as_u32(), 64);
        assert_eq!(p.as_i64(), 64);
        assert_eq!(p.as_u64(), 64);
        assert_eq!(p.as_i128(), 64);
        assert_eq!(p.as_u128(), 64);
        assert_eq!(p.as_isize(), 64);
        assert_eq!(p.as_usize(), 64);
    }

    #[test]
    fn unb_pow2_as_int_max_fitting() {
        assert_eq!(UnboundedPow2::from_exponent(6).as_i8(), 1 << 6);
        assert_eq!(UnboundedPow2::from_exponent(7).as_u8(), 1 << 7);
        assert_eq!(UnboundedPow2::from_exponent(14).as_i16(), 1 << 14);
        assert_eq!(UnboundedPow2::from_exponent(15).as_u16(), 1 << 15);
        assert_eq!(UnboundedPow2::from_exponent(30).as_i32(), 1 << 30);
        assert_eq!(UnboundedPow2::from_exponent(31).as_u32(), 1 << 31);
        assert_eq!(UnboundedPow2::from_exponent(62).as_i64(), 1 << 62);
        assert_eq!(UnboundedPow2::from_exponent(63).as_u64(), 1 << 63);
        assert_eq!(UnboundedPow2::from_exponent(126).as_i128(), 1 << 126);
        assert_eq!(UnboundedPow2::from_exponent(127).as_u128(), 1 << 127);
    }

    // Relies on the `debug_assert!` in `as_*`, which is compiled out in release.
    #[cfg(debug_assertions)]
    #[test]
    fn unb_pow2_as_int_not_fitting() {
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(7).as_i8()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(8).as_u8()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(15).as_i16()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(16).as_u16()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(31).as_i32()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(32).as_u32()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(63).as_i64()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(64).as_u64()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(127).as_i128()).is_err());
        assert!(std::panic::catch_unwind(|| UnboundedPow2::from_exponent(128).as_u128()).is_err());
    }

    #[test]
    fn unb_pow2_to_int() {
        assert_eq!(i8::try_from(UnboundedPow2::from_exponent(6)), Ok(1 << 6));
        assert_eq!(
            i8::try_from(UnboundedPow2::from_exponent(7)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(u8::try_from(UnboundedPow2::from_exponent(7)), Ok(1 << 7));
        assert_eq!(
            u8::try_from(UnboundedPow2::from_exponent(8)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(i16::try_from(UnboundedPow2::from_exponent(14)), Ok(1 << 14));
        assert_eq!(
            i16::try_from(UnboundedPow2::from_exponent(15)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(u16::try_from(UnboundedPow2::from_exponent(15)), Ok(1 << 15));
        assert_eq!(
            u16::try_from(UnboundedPow2::from_exponent(16)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(i32::try_from(UnboundedPow2::from_exponent(30)), Ok(1 << 30));
        assert_eq!(
            i32::try_from(UnboundedPow2::from_exponent(31)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(u32::try_from(UnboundedPow2::from_exponent(31)), Ok(1 << 31));
        assert_eq!(
            u32::try_from(UnboundedPow2::from_exponent(32)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(i64::try_from(UnboundedPow2::from_exponent(62)), Ok(1 << 62));
        assert_eq!(
            i64::try_from(UnboundedPow2::from_exponent(63)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(u64::try_from(UnboundedPow2::from_exponent(63)), Ok(1 << 63));
        assert_eq!(
            u64::try_from(UnboundedPow2::from_exponent(64)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(
            i128::try_from(UnboundedPow2::from_exponent(126)),
            Ok(1 << 126)
        );
        assert_eq!(
            i128::try_from(UnboundedPow2::from_exponent(127)),
            Err(Pow2OutOfRange)
        );

        assert_eq!(
            u128::try_from(UnboundedPow2::from_exponent(127)),
            Ok(1 << 127)
        );
        assert_eq!(
            u128::try_from(UnboundedPow2::from_exponent(128)),
            Err(Pow2OutOfRange)
        );
    }

    #[test]
    fn pow2_from_exponent() {
        let v = Pow2::<u8>::from_exponent(7);
        assert!(v.is_ok());
        assert_eq!(v.unwrap().exponent(), 7);

        let v = Pow2::<u8>::from_exponent(8);
        assert!(v.is_err());

        let v = Pow2::<u128>::from_exponent(127);
        assert!(v.is_ok());
        assert_eq!(v.unwrap().exponent(), 127);

        let v = Pow2::<u128>::from_exponent(128);
        assert!(v.is_err());
    }

    #[test]
    fn pow2_value() {
        let v = Pow2::<u8>::from_exponent(0);
        assert!(v.is_ok());
        assert_eq!(v.unwrap().value(), 1);

        let v = Pow2::<u8>::from_exponent(7);
        assert!(v.is_ok());
        assert_eq!(v.unwrap().value(), 128);
    }

    #[test]
    fn unb_pow2_mul() {
        let lhs = UnboundedPow2::from_exponent(6);
        let rhs = UnboundedPow2::from_exponent(7);
        assert_eq!(lhs * rhs, UnboundedPow2::from_exponent(13));
        assert_eq!(rhs * lhs, UnboundedPow2::from_exponent(13));
    }

    #[test]
    fn pow2_from_other() {
        assert_eq!(
            Pow2::<u16>::from(Pow2::<u8>::from_exponent(7).unwrap()),
            Pow2::<u16>::from_exponent(7).unwrap()
        );
        assert_eq!(
            Pow2::<u32>::from(Pow2::<u8>::from_exponent(7).unwrap()),
            Pow2::<u32>::from_exponent(7).unwrap()
        );
        assert_eq!(
            Pow2::<u64>::from(Pow2::<u8>::from_exponent(7).unwrap()),
            Pow2::<u64>::from_exponent(7).unwrap()
        );
        assert_eq!(
            Pow2::<u64>::from(Pow2::<u32>::from_exponent(31).unwrap()),
            Pow2::<u64>::from_exponent(31).unwrap()
        );

        /* // should not compile
        assert_eq!(Pow2::<u32>::from(Pow2::<u64>::from_exponent(31).unwrap()), Pow2::<u32>::from_exponent(31).unwrap());
        */
    }

    #[test]
    fn pow2_try_from_other() {
        assert_eq!(
            Pow2::<u8>::try_from(Pow2::<u16>::from_exponent(7).unwrap()),
            Ok(Pow2::<u8>::from_exponent(7).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(Pow2::<u16>::from_exponent(8).unwrap()),
            Err(Pow2OutOfRange)
        );
        assert_eq!(
            Pow2::<u8>::try_from(Pow2::<u32>::from_exponent(7).unwrap()),
            Ok(Pow2::<u8>::from_exponent(7).unwrap())
        );
        assert_eq!(
            Pow2::<u8>::try_from(Pow2::<u64>::from_exponent(7).unwrap()),
            Ok(Pow2::<u8>::from_exponent(7).unwrap())
        );
        assert_eq!(
            Pow2::<u32>::try_from(Pow2::<u128>::from_exponent(31).unwrap()),
            Ok(Pow2::<u32>::from_exponent(31).unwrap())
        );
        assert_eq!(
            Pow2::<u32>::try_from(Pow2::<u128>::from_exponent(32).unwrap()),
            Err(Pow2OutOfRange)
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_mul_overflow() {
        let lhs = UnboundedPow2::from_exponent(128);
        let rhs = UnboundedPow2::from_exponent(128);
        let _ = lhs * rhs;
    }

    #[test]
    fn unb_pow2_mul_assign() {
        let lhs = UnboundedPow2::from_exponent(6);
        let rhs = UnboundedPow2::from_exponent(7);
        let mut new_lhs = lhs;
        new_lhs *= rhs;
        assert_eq!(new_lhs, UnboundedPow2::from_exponent(13));

        let mut new_rhs = rhs;
        new_rhs *= lhs;
        assert_eq!(new_rhs, UnboundedPow2::from_exponent(13));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_mul_assign_overflow() {
        let mut lhs = UnboundedPow2::from_exponent(128);
        let rhs = UnboundedPow2::from_exponent(128);
        lhs *= rhs;
    }

    #[test]
    fn unb_pow2_mul_zero() {
        let lhs = UnboundedPow2::from_exponent(65);
        let rhs = UnboundedPow2::from_exponent(0);
        assert_eq!(lhs * rhs, lhs);
        assert_eq!(rhs * lhs, lhs);
    }

    #[test]
    fn unb_pow2_mul_assign_zero() {
        let lhs = UnboundedPow2::from_exponent(65);
        let rhs = UnboundedPow2::from_exponent(0);
        let mut new_lhs = lhs;
        new_lhs *= rhs;
        assert_eq!(new_lhs, lhs);

        let mut new_rhs = rhs;
        new_rhs *= lhs;
        assert_eq!(new_rhs, lhs);
    }

    #[test]
    fn unb_pow2_checked_mul() {
        assert_eq!(
            UnboundedPow2::from_exponent(12).checked_mul(UnboundedPow2::from_exponent(13)),
            Some(UnboundedPow2::from_exponent(25))
        );
        assert_eq!(
            UnboundedPow2::from_exponent(12).checked_mul(UnboundedPow2::from_exponent(0)),
            Some(UnboundedPow2::from_exponent(12))
        );
        assert_eq!(
            UnboundedPow2::from_exponent(0).checked_mul(UnboundedPow2::from_exponent(13)),
            Some(UnboundedPow2::from_exponent(13))
        );
    }

    #[test]
    fn unb_pow2_checked_mul_boundary() {
        assert_eq!(
            UnboundedPow2::from_exponent(254).checked_mul(UnboundedPow2::from_exponent(1)),
            Some(UnboundedPow2::from_exponent(255))
        );
        assert_eq!(
            UnboundedPow2::from_exponent(254).checked_mul(UnboundedPow2::from_exponent(2)),
            None
        );
    }

    #[test]
    fn pow2_checked_mul_boundary() {
        assert_eq!(
            Pow2::<u32>::from_exponent(12)
                .unwrap()
                .checked_mul(Pow2::<u32>::from_exponent(19).unwrap()),
            Some(Pow2::<u32>::from_exponent(31).unwrap())
        );
        assert_eq!(
            Pow2::<u32>::from_exponent(12)
                .unwrap()
                .checked_mul(Pow2::<u32>::from_exponent(20).unwrap()),
            None
        );
    }

    #[test]
    fn unb_pow2_saturating_mul() {
        assert_eq!(
            UnboundedPow2::from_exponent(13).saturating_mul(UnboundedPow2::from_exponent(14)),
            UnboundedPow2::from_exponent(27)
        );
        assert_eq!(
            UnboundedPow2::from_exponent(13).saturating_mul(UnboundedPow2::from_exponent(0)),
            UnboundedPow2::from_exponent(13)
        );
        assert_eq!(
            UnboundedPow2::from_exponent(0).saturating_mul(UnboundedPow2::from_exponent(14)),
            UnboundedPow2::from_exponent(14)
        );
    }

    #[test]
    fn unb_pow2_saturating_mul_boundary() {
        assert_eq!(
            UnboundedPow2::from_exponent(254).saturating_mul(UnboundedPow2::from_exponent(1)),
            UnboundedPow2::from_exponent(255)
        );
        assert_eq!(
            UnboundedPow2::from_exponent(254).saturating_mul(UnboundedPow2::from_exponent(2)),
            UnboundedPow2::from_exponent(255)
        );
    }

    #[test]
    fn pow2_saturating_mul_boundary() {
        assert_eq!(
            Pow2::<u32>::from_exponent(12)
                .unwrap()
                .saturating_mul(Pow2::<u32>::from_exponent(18).unwrap()),
            Pow2::<u32>::from_exponent(30).unwrap()
        );
        assert_eq!(
            Pow2::<u32>::from_exponent(12)
                .unwrap()
                .saturating_mul(Pow2::<u32>::from_exponent(19).unwrap()),
            Pow2::<u32>::from_exponent(31).unwrap()
        );
        assert_eq!(
            Pow2::<u32>::from_exponent(12)
                .unwrap()
                .saturating_mul(Pow2::<u32>::from_exponent(20).unwrap()),
            Pow2::<u32>::from_exponent(31).unwrap()
        );
    }

    #[test]
    fn unb_pow2_div_positive() {
        let lhs = UnboundedPow2::from_exponent(6);
        let rhs = UnboundedPow2::from_exponent(3);
        assert_eq!(lhs / rhs, UnboundedPow2::from_exponent(3));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_div_overflow() {
        let lhs = UnboundedPow2::from_exponent(127);
        let rhs = UnboundedPow2::from_exponent(128);
        let _ = lhs / rhs;
    }

    #[test]
    fn unb_pow2_div_assign_positive() {
        let mut lhs = UnboundedPow2::from_exponent(6);
        let rhs = UnboundedPow2::from_exponent(3);
        lhs /= rhs;
        assert_eq!(lhs, UnboundedPow2::from_exponent(3));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_div_assign_overflow() {
        let mut lhs = UnboundedPow2::from_exponent(127);
        let rhs = UnboundedPow2::from_exponent(128);
        lhs /= rhs;
    }

    #[test]
    fn unb_pow2_checked_div_overflow() {
        let lhs = UnboundedPow2::from_exponent(3);
        let rhs = UnboundedPow2::from_exponent(6);
        assert_eq!(lhs.checked_div(rhs), None);
        assert_eq!(rhs.checked_div(lhs), Some(UnboundedPow2::from_exponent(3)));
    }

    #[test]
    fn pow2_checked_div_overflow() {
        let lhs = Pow2::<u32>::from_exponent(3).unwrap();
        let rhs = Pow2::<u32>::from_exponent(6).unwrap();
        assert_eq!(lhs.checked_div(rhs), None);
        assert_eq!(
            rhs.checked_div(lhs),
            Some(Pow2::<u32>::from_exponent(3).unwrap())
        );
    }

    #[test]
    fn unb_pow2_saturating_div_overflow() {
        let lhs = UnboundedPow2::from_exponent(3);
        let rhs = UnboundedPow2::from_exponent(6);
        assert_eq!(lhs.saturating_div(rhs), UnboundedPow2::from_exponent(0));
        assert_eq!(rhs.saturating_div(lhs), UnboundedPow2::from_exponent(3));
    }

    #[test]
    fn pow2_saturating_div_overflow() {
        let lhs = Pow2::<u32>::from_exponent(3).unwrap();
        let rhs = Pow2::<u32>::from_exponent(6).unwrap();
        assert_eq!(
            lhs.saturating_div(rhs),
            Pow2::<u32>::from_exponent(0).unwrap()
        );
        assert_eq!(
            rhs.saturating_div(lhs),
            Pow2::<u32>::from_exponent(3).unwrap()
        );
    }

    #[test]
    fn unb_pow2_div_one() {
        let lhs = UnboundedPow2::from_exponent(3);
        let rhs = UnboundedPow2::from_exponent(0);
        assert_eq!(lhs / rhs, lhs);
    }

    #[test]
    fn unb_pow2_div_assign_one() {
        let lhs = UnboundedPow2::from_exponent(3);
        let rhs = UnboundedPow2::from_exponent(0);
        let mut new_lhs = lhs;
        new_lhs /= rhs;
        assert_eq!(new_lhs, lhs);
    }

    #[test]
    fn unb_pow2_div_self() {
        let v = UnboundedPow2::from_exponent(123);
        assert_eq!(v / v, UnboundedPow2::from_exponent(0));
    }

    #[test]
    fn unb_pow2_div_assign_self() {
        let mut v = UnboundedPow2::from_exponent(123);
        v /= v;
        assert_eq!(v, UnboundedPow2::from_exponent(0));
    }

    #[test]
    fn unb_pow2_int_mul_one() {
        assert_eq!(123 * UnboundedPow2::from_exponent(0), 123);
        assert_eq!(-123 * UnboundedPow2::from_exponent(0), -123);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_int_mul_overflow() {
        let _ = 123_i32 * UnboundedPow2::from_exponent(27);
    }

    #[test]
    fn unb_pow2_int_mul_assign_one() {
        let mut v = 123;
        v *= UnboundedPow2::from_exponent(0);
        assert_eq!(v, 123);

        let mut v = -123;
        v *= UnboundedPow2::from_exponent(0);
        assert_eq!(v, -123);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_int_mul_assign_overflow() {
        let mut lhs = 123_i32;
        lhs *= UnboundedPow2::from_exponent(27);
    }

    #[test]
    fn unb_pow2_int_mul() {
        assert_eq!(123_i32 * UnboundedPow2::from_exponent(15), 123 << 15);
        assert_eq!(-123_i32 * UnboundedPow2::from_exponent(15), -123 << 15);
        assert_eq!(123_i64 * UnboundedPow2::from_exponent(32), 123 << 32);
        assert_eq!(-123_i64 * UnboundedPow2::from_exponent(32), -123 << 32);
        assert_eq!(123_u64 * UnboundedPow2::from_exponent(46), 123 << 46);
    }

    #[test]
    fn unb_pow2_int_mul_assign() {
        let mut v = 123_i32;
        v *= UnboundedPow2::from_exponent(15);
        assert_eq!(v, 123 << 15);

        let mut v = -123_i32;
        v *= UnboundedPow2::from_exponent(15);
        assert_eq!(v, -123 << 15);

        let mut v = 123_i64;
        v *= UnboundedPow2::from_exponent(32);
        assert_eq!(v, 123 << 32);

        let mut v = -123_i64;
        v *= UnboundedPow2::from_exponent(32);
        assert_eq!(v, -123 << 32);

        let mut v = 123_u64;
        v *= UnboundedPow2::from_exponent(46);
        assert_eq!(v, 123 << 46);
    }

    #[test]
    fn unb_pow2_int_checked_mul_boundary() {
        assert_eq!(
            checked_mul(i32::MAX as u32 + 1, UnboundedPow2::from_exponent(1)),
            None
        );
        assert_eq!(
            checked_mul(i32::MAX as u32, UnboundedPow2::from_exponent(1)),
            Some(u32::MAX - 1)
        );
    }

    #[test]
    fn pow2_int_checked_mul_boundary() {
        assert_eq!(
            checked_mul(i32::MAX as u32 + 1, Pow2::<u32>::from_exponent(1).unwrap()),
            None
        );
        assert_eq!(
            checked_mul(i32::MAX as u32 + 1, Pow2::<u8>::from_exponent(1).unwrap()),
            None
        );
        assert_eq!(
            checked_mul(i32::MAX as u32, Pow2::<u32>::from_exponent(1).unwrap()),
            Some(u32::MAX - 1)
        );

        /* // Should not compile.
        assert_eq!(
            checked_mul(i32::MAX as u32, Pow2::<u64>::from_exponent(1).unwrap()),
            Some(u32::MAX - 1)
        );
        */
    }

    #[test]
    fn unb_pow2_int_div_exact() {
        assert_eq!(32u64 / UnboundedPow2::from_exponent(5), 1);
        assert_eq!(64u64 / UnboundedPow2::from_exponent(3), 8);
        assert_eq!(-64i64 / UnboundedPow2::from_exponent(3), -8);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_int_div_divisor_out_of_range() {
        let _ = 123_i32 / UnboundedPow2::from_exponent(32);
    }

    #[test]
    fn unb_pow2_int_div_assign_exact() {
        let mut v = 32u64;
        v /= UnboundedPow2::from_exponent(5);
        assert_eq!(v, 1);

        let mut v = 64u64;
        v /= UnboundedPow2::from_exponent(3);
        assert_eq!(v, 8);

        let mut v = -64i64;
        v /= UnboundedPow2::from_exponent(3);
        assert_eq!(v, -8);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_int_div_assign_divisor_out_of_range() {
        let _ = 123_i32 / UnboundedPow2::from_exponent(32);
    }

    #[test]
    fn unb_pow2_int_div_rounds_towards_zero() {
        assert_eq!(37u64 / UnboundedPow2::from_exponent(5), 1);
        assert_eq!(63u64 / UnboundedPow2::from_exponent(5), 1);
        assert_eq!(-37i64 / UnboundedPow2::from_exponent(5), -1);
        assert_eq!(-1i64 / UnboundedPow2::from_exponent(5), 0);
    }

    #[test]
    fn pow2_int_div_rounds_towards_zero() {
        assert_eq!(37u64 / Pow2::<u8>::from_exponent(5).unwrap(), 1);
        assert_eq!(63u64 / Pow2::<u8>::from_exponent(5).unwrap(), 1);
        assert_eq!(-37i64 / Pow2::<u8>::from_exponent(5).unwrap(), -1);
        assert_eq!(-1i64 / Pow2::<u8>::from_exponent(5).unwrap(), 0);
    }

    #[test]
    fn unb_pow2_int_div_assign_rounds_towards_zero() {
        let mut v = 37u64;
        v /= UnboundedPow2::from_exponent(5);
        assert_eq!(v, 1);

        let mut v = 63u64;
        v /= UnboundedPow2::from_exponent(5);
        assert_eq!(v, 1);

        let mut v = -37i64;
        v /= UnboundedPow2::from_exponent(5);
        assert_eq!(v, -1);

        let mut v = -1i64;
        v /= UnboundedPow2::from_exponent(5);
        assert_eq!(v, 0);
    }

    #[test]
    fn pow2_int_div_assign_rounds_towards_zero() {
        let mut v = 37u64;
        v /= Pow2::<u8>::from_exponent(5).unwrap();
        assert_eq!(v, 1);

        let mut v = 63u64;
        v /= Pow2::<u8>::from_exponent(5).unwrap();
        assert_eq!(v, 1);

        let mut v = -37i64;
        v /= Pow2::<u8>::from_exponent(5).unwrap();
        assert_eq!(v, -1);

        let mut v = -1i64;
        v /= Pow2::<u8>::from_exponent(5).unwrap();
        assert_eq!(v, 0);
    }

    #[test]
    fn unb_pow2_int_div_by_one_is_identity() {
        assert_eq!(42i64 / UnboundedPow2::from_exponent(0), 42);
        assert_eq!(-42i64 / UnboundedPow2::from_exponent(0), -42);
    }

    #[test]
    fn unb_pow2_int_div_assign_by_one_is_identity() {
        let mut v = 42i64;
        v /= UnboundedPow2::from_exponent(0);
        assert_eq!(v, 42);

        let mut v = -42i64;
        v /= UnboundedPow2::from_exponent(0);
        assert_eq!(v, -42);
    }

    #[test]
    fn unb_pow2_int_div_min() {
        assert_eq!(i32::MIN / UnboundedPow2::from_exponent(30), -2);
        assert_eq!(i32::MIN / UnboundedPow2::from_exponent(31), -1);
    }

    #[test]
    fn unb_pow2_int_div_assign_min() {
        let mut v = i32::MIN;
        v /= UnboundedPow2::from_exponent(30);
        assert_eq!(v, -2);

        let mut v = i32::MIN;
        v /= UnboundedPow2::from_exponent(31);
        assert_eq!(v, -1);
    }

    #[test]
    fn unb_pow2_int_div_max() {
        assert_eq!(i32::MAX / UnboundedPow2::from_exponent(30), 1);
        assert_eq!(i32::MAX / UnboundedPow2::from_exponent(31), 0);
        assert_eq!(u32::MAX / UnboundedPow2::from_exponent(31), 1);
    }

    #[test]
    fn unb_pow2_int_div_assign_max() {
        let mut v = i32::MAX;
        v /= UnboundedPow2::from_exponent(30);
        assert_eq!(v, 1);

        let mut v = i32::MAX;
        v /= UnboundedPow2::from_exponent(31);
        assert_eq!(v, 0);

        let mut v = u32::MAX;
        v /= UnboundedPow2::from_exponent(31);
        assert_eq!(v, 1);
    }

    #[test]
    fn unb_pow2_checked_div_boundary() {
        assert_eq!(
            checked_div(i32::MAX, UnboundedPow2::from_exponent(30)),
            Some(1)
        );
        assert_eq!(
            checked_div(i32::MIN, UnboundedPow2::from_exponent(30)),
            Some(-2)
        );
        assert_eq!(
            checked_div(123_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div(-123_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(checked_div(123_i32, UnboundedPow2::from_exponent(32)), None);
        assert_eq!(
            checked_div(-123_i32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_div_boundary() {
        assert_eq!(unbounded_div(i32::MAX, UnboundedPow2::from_exponent(30)), 1);
        assert_eq!(unbounded_div(i32::MAX, UnboundedPow2::from_exponent(31)), 0);
        assert_eq!(
            unbounded_div(i32::MIN, UnboundedPow2::from_exponent(30)),
            -2
        );
        assert_eq!(
            unbounded_div(i32::MIN, UnboundedPow2::from_exponent(31)),
            -1
        );
        assert_eq!(unbounded_div(i32::MIN, UnboundedPow2::from_exponent(32)), 0);
        assert_eq!(unbounded_div(123_i32, UnboundedPow2::from_exponent(31)), 0);
        assert_eq!(unbounded_div(-123_i32, UnboundedPow2::from_exponent(31)), 0);
        assert_eq!(unbounded_div(123_i32, UnboundedPow2::from_exponent(32)), 0);
        assert_eq!(unbounded_div(-123_i32, UnboundedPow2::from_exponent(32)), 0);
    }

    #[test]
    fn unb_pow2_rem() {
        assert_eq!(130i32 % UnboundedPow2::from_exponent(4), 2);
        assert_eq!(i32::MIN % UnboundedPow2::from_exponent(4), 0);
        assert_eq!((i32::MIN + 1) % UnboundedPow2::from_exponent(4), -15);
        assert_eq!(i32::MAX % UnboundedPow2::from_exponent(4), 15);
    }

    #[test]
    fn unb_pow2_rem_assign() {
        let mut v = 130i32;
        v %= UnboundedPow2::from_exponent(4);
        assert_eq!(v, 2);

        let mut v = i32::MIN;
        v %= UnboundedPow2::from_exponent(4);
        assert_eq!(v, 0);

        let mut v = i32::MIN + 1;
        v %= UnboundedPow2::from_exponent(4);
        assert_eq!(v, -15);

        let mut v = i32::MAX;
        v %= UnboundedPow2::from_exponent(4);
        assert_eq!(v, 15);
    }

    #[test]
    fn unb_pow2_checked_rem() {
        assert_eq!(
            checked_rem(130i32, UnboundedPow2::from_exponent(4)),
            Some(2)
        );
        assert_eq!(
            checked_rem(i32::MIN, UnboundedPow2::from_exponent(4)),
            Some(0)
        );
        assert_eq!(
            checked_rem(i32::MIN + 1, UnboundedPow2::from_exponent(4)),
            Some(-15)
        );
        assert_eq!(
            checked_rem(i32::MAX, UnboundedPow2::from_exponent(4)),
            Some(15)
        );
        assert_eq!(
            checked_rem(i32::MAX, UnboundedPow2::from_exponent(31)),
            Some(i32::MAX)
        );
        assert_eq!(
            checked_rem(i32::MAX, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_rem() {
        assert_eq!(unbounded_rem(130i32, UnboundedPow2::from_exponent(4)), 2);
        assert_eq!(unbounded_rem(i32::MIN, UnboundedPow2::from_exponent(4)), 0);
        assert_eq!(
            unbounded_rem(i32::MIN + 1, UnboundedPow2::from_exponent(4)),
            -15
        );
        assert_eq!(unbounded_rem(i32::MAX, UnboundedPow2::from_exponent(4)), 15);
        assert_eq!(
            unbounded_rem(i32::MAX, UnboundedPow2::from_exponent(31)),
            i32::MAX
        );
        assert_eq!(
            unbounded_rem(i32::MAX, UnboundedPow2::from_exponent(32)),
            i32::MAX
        );
        assert_eq!(unbounded_rem(i32::MIN, UnboundedPow2::from_exponent(31)), 0);
        assert_eq!(
            unbounded_rem(i32::MIN, UnboundedPow2::from_exponent(32)),
            i32::MIN
        );
    }

    #[test]
    fn pow2_rem() {
        assert_eq!(130i32 % Pow2::<u8>::from_exponent(4).unwrap(), 2);
        assert_eq!(i32::MIN % Pow2::<u8>::from_exponent(4).unwrap(), 0);
        assert_eq!((i32::MIN + 1) % Pow2::<u8>::from_exponent(4).unwrap(), -15);
        assert_eq!(i32::MAX % Pow2::<u8>::from_exponent(4).unwrap(), 15);
    }

    #[test]
    fn pow2_rem_assign() {
        let mut v = 130i32;
        v %= Pow2::<u8>::from_exponent(4).unwrap();
        assert_eq!(v, 2);

        let mut v = i32::MIN;
        v %= Pow2::<u8>::from_exponent(4).unwrap();
        assert_eq!(v, 0);

        let mut v = i32::MIN + 1;
        v %= Pow2::<u8>::from_exponent(4).unwrap();
        assert_eq!(v, -15);

        let mut v = i32::MAX;
        v %= Pow2::<u8>::from_exponent(4).unwrap();
        assert_eq!(v, 15);
    }

    #[test]
    fn unb_pow2_div_floor_u64_exact() {
        assert_eq!(div_floor(32u64, UnboundedPow2::from_exponent(5)), 1);
        assert_eq!(div_floor(64u64, UnboundedPow2::from_exponent(3)), 8);
    }

    #[test]
    fn pow2_div_floor_u64_exact() {
        assert_eq!(div_floor(32u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
        assert_eq!(div_floor(64u64, Pow2::<u8>::from_exponent(3).unwrap()), 8);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_div_floor_divisor_out_of_range() {
        let _ = div_floor(32u64, UnboundedPow2::from_exponent(64));
    }

    #[test]
    fn unb_pow2_div_floor_u64_rounds_down() {
        assert_eq!(div_floor(37u64, UnboundedPow2::from_exponent(5)), 1);
        assert_eq!(div_floor(63u64, UnboundedPow2::from_exponent(5)), 1);
    }

    #[test]
    fn pow2_div_floor_u64_rounds_down() {
        assert_eq!(div_floor(37u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
        assert_eq!(div_floor(63u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
    }

    #[test]
    fn unb_pow2_div_floor_u64_zero() {
        assert_eq!(div_floor(0u64, UnboundedPow2::from_exponent(5)), 0);
    }

    #[test]
    fn pow2_div_floor_u64_zero() {
        assert_eq!(div_floor(0u64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
    }

    #[test]
    fn unb_pow2_div_floor_i64_positive() {
        assert_eq!(div_floor(37i64, UnboundedPow2::from_exponent(5)), 1);
    }

    #[test]
    fn pow2_div_floor_i64_positive() {
        assert_eq!(div_floor(37i64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
    }

    #[test]
    fn unb_pow2_div_floor_i64_exact_negative() {
        assert_eq!(div_floor(-32i64, UnboundedPow2::from_exponent(5)), -1);
    }

    #[test]
    fn pow2_div_floor_i64_exact_negative() {
        assert_eq!(div_floor(-32i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
    }

    #[test]
    fn unb_pow2_div_floor_i64_negative_rounds_toward_neg_inf() {
        assert_eq!(div_floor(-37i64, UnboundedPow2::from_exponent(5)), -2);
        assert_eq!(div_floor(-1i64, UnboundedPow2::from_exponent(5)), -1);
    }

    #[test]
    fn pow2_div_floor_i64_negative_rounds_toward_neg_inf() {
        assert_eq!(div_floor(-37i64, Pow2::<u8>::from_exponent(5).unwrap()), -2);
        assert_eq!(div_floor(-1i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
    }

    #[test]
    fn unb_pow2_div_floor_by_one_is_identity() {
        assert_eq!(div_floor(42i64, UnboundedPow2::from_exponent(0)), 42);
        assert_eq!(div_floor(-42i64, UnboundedPow2::from_exponent(0)), -42);
    }

    #[test]
    fn pow2_div_floor_by_one_is_identity() {
        assert_eq!(div_floor(42i64, Pow2::<u8>::from_exponent(0).unwrap()), 42);
        assert_eq!(
            div_floor(-42i64, Pow2::<u8>::from_exponent(0).unwrap()),
            -42
        );
    }

    #[test]
    fn unb_pow2_div_floor_min() {
        assert_eq!(div_floor(i32::MIN, UnboundedPow2::from_exponent(30)), -2);
        assert_eq!(div_floor(i32::MIN, UnboundedPow2::from_exponent(31)), -1);
    }

    #[test]
    fn pow2_div_floor_min() {
        assert_eq!(
            div_floor(i32::MIN, Pow2::<u32>::from_exponent(30).unwrap()),
            -2
        );
        assert_eq!(
            div_floor(i32::MIN, Pow2::<u32>::from_exponent(31).unwrap()),
            -1
        );
    }

    #[test]
    fn unb_pow2_div_floor_max() {
        assert_eq!(div_floor(i32::MAX, UnboundedPow2::from_exponent(30)), 1);
        assert_eq!(div_floor(i32::MAX, UnboundedPow2::from_exponent(31)), 0);
        assert_eq!(div_floor(u32::MAX, UnboundedPow2::from_exponent(31)), 1);
    }

    #[test]
    fn pow2_div_floor_max() {
        assert_eq!(
            div_floor(i32::MAX, Pow2::<u32>::from_exponent(30).unwrap()),
            1
        );
        assert_eq!(
            div_floor(i32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            0
        );
        assert_eq!(
            div_floor(u32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            1
        );
    }

    #[test]
    fn unb_pow2_checked_div_floor_boundary() {
        assert_eq!(
            checked_div_floor(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_div_floor(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div_floor(0_i32, UnboundedPow2::from_exponent(32)),
            None
        );

        assert_eq!(
            checked_div_floor(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div_floor(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_div_floor_boundary() {
        assert_eq!(
            unbounded_div_floor(0_i32, UnboundedPow2::from_exponent(30)),
            0
        );
        assert_eq!(
            unbounded_div_floor(i32::MIN, UnboundedPow2::from_exponent(30)),
            -2
        );
        assert_eq!(
            unbounded_div_floor(0_i32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_div_floor(-1_i32, UnboundedPow2::from_exponent(31)),
            -1
        );
        assert_eq!(
            unbounded_div_floor(i32::MIN, UnboundedPow2::from_exponent(31)),
            -1
        );
        assert_eq!(
            unbounded_div_floor(0_i32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_floor(-1_i32, UnboundedPow2::from_exponent(32)),
            -1
        );
        assert_eq!(
            unbounded_div_floor(0_i32, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_floor(i32::MAX, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_floor(i32::MIN, UnboundedPow2::from_exponent(255)),
            -1
        );

        assert_eq!(
            unbounded_div_floor(0_u32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_div_floor(u32::MAX, UnboundedPow2::from_exponent(31)),
            1
        );
        assert_eq!(
            unbounded_div_floor(0_u32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_floor(u32::MAX, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_floor(0_u32, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_floor(u32::MAX, UnboundedPow2::from_exponent(255)),
            0
        );
    }

    #[test]
    fn unb_pow2_div_ceil_u64_exact() {
        assert_eq!(div_ceil(32u64, UnboundedPow2::from_exponent(5)), 1);
    }

    #[test]
    fn pow2_div_ceil_u64_exact() {
        assert_eq!(div_ceil(32u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_div_ceil_divisor_out_of_range() {
        let _ = div_ceil(32u64, UnboundedPow2::from_exponent(64));
    }

    #[test]
    fn unb_pow2_ceil_u64_rounds_up() {
        assert_eq!(div_ceil(33u64, UnboundedPow2::from_exponent(5)), 2);
        assert_eq!(div_ceil(63u64, UnboundedPow2::from_exponent(5)), 2);
    }

    #[test]
    fn pow2_div_ceil_u64_rounds_up() {
        assert_eq!(div_ceil(33u64, Pow2::<u8>::from_exponent(5).unwrap()), 2);
        assert_eq!(div_ceil(63u64, Pow2::<u8>::from_exponent(5).unwrap()), 2);
    }

    #[test]
    fn unb_pow2_div_ceil_u64_zero() {
        assert_eq!(div_ceil(0u64, UnboundedPow2::from_exponent(5)), 0);
    }

    #[test]
    fn pow2_div_ceil_u64_zero() {
        assert_eq!(div_ceil(0u64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
    }

    #[test]
    fn unb_pow2_div_ceil_i64_negative() {
        assert_eq!(div_ceil(-37i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_ceil(-32i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_ceil(-1i64, UnboundedPow2::from_exponent(5)), 0);
        assert_eq!(div_ceil(-33i64, UnboundedPow2::from_exponent(5)), -1);
    }

    #[test]
    fn pow2_div_ceil_i64_negative() {
        assert_eq!(div_ceil(-37i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_ceil(-32i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_ceil(-1i64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
        assert_eq!(div_ceil(-33i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
    }

    #[test]
    fn unb_pow2_div_ceil_by_one_is_identity() {
        assert_eq!(div_ceil(42i64, UnboundedPow2::from_exponent(0)), 42);
        assert_eq!(div_ceil(-42i64, UnboundedPow2::from_exponent(0)), -42);
    }

    #[test]
    fn pow2_div_ceil_by_one_is_identity() {
        assert_eq!(div_ceil(42i64, Pow2::<u8>::from_exponent(0).unwrap()), 42);
        assert_eq!(div_ceil(-42i64, Pow2::<u8>::from_exponent(0).unwrap()), -42);
    }

    #[test]
    fn unb_pow2_div_ceil_min() {
        assert_eq!(div_ceil(i32::MIN, UnboundedPow2::from_exponent(30)), -2);
        assert_eq!(div_ceil(i32::MIN, UnboundedPow2::from_exponent(31)), -1);
        assert_eq!(div_ceil(i64::MIN, UnboundedPow2::from_exponent(62)), -2);
        assert_eq!(div_ceil(i64::MIN, UnboundedPow2::from_exponent(63)), -1);
    }

    #[test]
    fn pow2_div_ceil_min() {
        assert_eq!(
            div_ceil(i32::MIN, Pow2::<u32>::from_exponent(30).unwrap()),
            -2
        );
        assert_eq!(
            div_ceil(i32::MIN, Pow2::<u32>::from_exponent(31).unwrap()),
            -1
        );
        assert_eq!(
            div_ceil(i64::MIN, Pow2::<u64>::from_exponent(62).unwrap()),
            -2
        );
        assert_eq!(
            div_ceil(i64::MIN, Pow2::<u64>::from_exponent(63).unwrap()),
            -1
        );
    }

    #[test]
    fn unb_pow2_div_ceil_max() {
        assert_eq!(div_ceil(i32::MAX, UnboundedPow2::from_exponent(30)), 2);
        assert_eq!(div_ceil(i32::MAX, UnboundedPow2::from_exponent(31)), 1);
        assert_eq!(div_ceil(u32::MAX, UnboundedPow2::from_exponent(31)), 2);
        assert_eq!(div_ceil(i64::MAX, UnboundedPow2::from_exponent(62)), 2);
        assert_eq!(div_ceil(i64::MAX, UnboundedPow2::from_exponent(63)), 1);
        assert_eq!(div_ceil(u64::MAX, UnboundedPow2::from_exponent(63)), 2);
    }

    #[test]
    fn pow2_div_ceil_max() {
        assert_eq!(
            div_ceil(i32::MAX, Pow2::<u32>::from_exponent(30).unwrap()),
            2
        );
        assert_eq!(
            div_ceil(i32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            1
        );
        assert_eq!(
            div_ceil(u32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            2
        );
        assert_eq!(
            div_ceil(i64::MAX, Pow2::<u64>::from_exponent(62).unwrap()),
            2
        );
        assert_eq!(
            div_ceil(i64::MAX, Pow2::<u64>::from_exponent(63).unwrap()),
            1
        );
        assert_eq!(
            div_ceil(u64::MAX, Pow2::<u64>::from_exponent(63).unwrap()),
            2
        );
    }

    #[test]
    fn unb_pow2_checked_div_ceil_boundary() {
        assert_eq!(
            checked_div_ceil(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_div_ceil(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div_ceil(0_i32, UnboundedPow2::from_exponent(32)),
            None
        );

        assert_eq!(
            checked_div_ceil(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div_ceil(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_div_ceil_boundary() {
        assert_eq!(
            unbounded_div_ceil(0_i32, UnboundedPow2::from_exponent(30)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(i32::MAX, UnboundedPow2::from_exponent(30)),
            2
        );
        assert_eq!(
            unbounded_div_ceil(0_i32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(i32::MIN, UnboundedPow2::from_exponent(31)),
            -1
        );
        assert_eq!(
            unbounded_div_ceil(-1_i32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(0_i32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(1_i32, UnboundedPow2::from_exponent(32)),
            1
        );
        assert_eq!(
            unbounded_div_ceil(i32::MAX, UnboundedPow2::from_exponent(32)),
            1
        );
        assert_eq!(
            unbounded_div_ceil(-1_i32, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(i32::MIN, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(i32::MAX, UnboundedPow2::from_exponent(255)),
            1
        );

        assert_eq!(
            unbounded_div_ceil(0_u32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(0_u32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(0_u32, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_ceil(u32::MAX, UnboundedPow2::from_exponent(255)),
            1
        );
    }

    #[test]
    fn unb_pow2_div_round_u64_exact() {
        assert_eq!(div_round(32u64, UnboundedPow2::from_exponent(5)), 1);
    }

    #[test]
    fn pow2_div_round_u64_exact() {
        assert_eq!(div_round(32u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_div_round_divisor_out_of_range() {
        let _ = div_round(32u64, UnboundedPow2::from_exponent(64));
    }

    #[test]
    fn unb_pow2_div_round_u64_rounds() {
        assert_eq!(div_round(17u64, UnboundedPow2::from_exponent(5)), 1);
        assert_eq!(div_round(16u64, UnboundedPow2::from_exponent(5)), 1);
        assert_eq!(div_round(15u64, UnboundedPow2::from_exponent(5)), 0);
        assert_eq!(div_round(33u64, UnboundedPow2::from_exponent(5)), 1);
        assert_eq!(div_round(48u64, UnboundedPow2::from_exponent(5)), 2);
        assert_eq!(div_round(47u64, UnboundedPow2::from_exponent(5)), 1);
    }

    #[test]
    fn pow2_div_round_u64_rounds() {
        assert_eq!(div_round(17u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
        assert_eq!(div_round(16u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
        assert_eq!(div_round(15u64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
        assert_eq!(div_round(33u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
        assert_eq!(div_round(48u64, Pow2::<u8>::from_exponent(5).unwrap()), 2);
        assert_eq!(div_round(47u64, Pow2::<u8>::from_exponent(5).unwrap()), 1);
    }

    #[test]
    fn unb_pow2_div_round_u64_zero() {
        assert_eq!(div_round(0u64, UnboundedPow2::from_exponent(5)), 0);
    }

    #[test]
    fn pow2_div_round_u64_zero() {
        assert_eq!(div_round(0u64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
    }

    #[test]
    fn unb_pow2_div_round_i64_negative() {
        assert_eq!(div_round(-37i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_round(-32i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_round(-16i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_round(-17i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_round(-15i64, UnboundedPow2::from_exponent(5)), 0);
        assert_eq!(div_round(-1i64, UnboundedPow2::from_exponent(5)), 0);
        assert_eq!(div_round(-33i64, UnboundedPow2::from_exponent(5)), -1);
        assert_eq!(div_round(-48i64, UnboundedPow2::from_exponent(5)), -2);
        assert_eq!(div_round(-47i64, UnboundedPow2::from_exponent(5)), -1);
    }

    #[test]
    fn pow2_div_round_i64_negative() {
        assert_eq!(div_round(-37i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_round(-32i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_round(-16i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_round(-17i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_round(-15i64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
        assert_eq!(div_round(-1i64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
        assert_eq!(div_round(-33i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
        assert_eq!(div_round(-48i64, Pow2::<u8>::from_exponent(5).unwrap()), -2);
        assert_eq!(div_round(-47i64, Pow2::<u8>::from_exponent(5).unwrap()), -1);
    }

    #[test]
    fn unb_pow2_div_round_by_one_is_identity() {
        assert_eq!(div_round(42i64, UnboundedPow2::from_exponent(0)), 42);
        assert_eq!(div_round(-42i64, UnboundedPow2::from_exponent(0)), -42);
    }

    #[test]
    fn pow2_div_round_by_one_is_identity() {
        assert_eq!(div_round(42i64, Pow2::<u8>::from_exponent(0).unwrap()), 42);
        assert_eq!(
            div_round(-42i64, Pow2::<u8>::from_exponent(0).unwrap()),
            -42
        );
    }

    #[test]
    fn unb_pow2_div_round_min() {
        assert_eq!(div_round(i32::MIN, UnboundedPow2::from_exponent(30)), -2);
        assert_eq!(div_round(i32::MIN, UnboundedPow2::from_exponent(31)), -1);
        assert_eq!(div_round(i64::MIN, UnboundedPow2::from_exponent(62)), -2);
        assert_eq!(div_round(i64::MIN, UnboundedPow2::from_exponent(63)), -1);
    }

    #[test]
    fn pow2_div_round_min() {
        assert_eq!(
            div_round(i32::MIN, Pow2::<u32>::from_exponent(30).unwrap()),
            -2
        );
        assert_eq!(
            div_round(i32::MIN, Pow2::<u32>::from_exponent(31).unwrap()),
            -1
        );
        assert_eq!(
            div_round(i64::MIN, Pow2::<u64>::from_exponent(62).unwrap()),
            -2
        );
        assert_eq!(
            div_round(i64::MIN, Pow2::<u64>::from_exponent(63).unwrap()),
            -1
        );
    }

    #[test]
    fn unb_pow2_div_round_max() {
        assert_eq!(div_round(i32::MAX, UnboundedPow2::from_exponent(30)), 2);
        assert_eq!(div_round(i32::MAX, UnboundedPow2::from_exponent(31)), 1);
        assert_eq!(div_round(u32::MAX, UnboundedPow2::from_exponent(31)), 2);
        assert_eq!(div_round(i64::MAX, UnboundedPow2::from_exponent(62)), 2);
        assert_eq!(div_round(i64::MAX, UnboundedPow2::from_exponent(63)), 1);
        assert_eq!(div_round(u64::MAX, UnboundedPow2::from_exponent(63)), 2);
    }

    #[test]
    fn pow2_div_round_max() {
        assert_eq!(
            div_round(i32::MAX, Pow2::<u32>::from_exponent(30).unwrap()),
            2
        );
        assert_eq!(
            div_round(i32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            1
        );
        assert_eq!(
            div_round(u32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            2
        );
        assert_eq!(
            div_round(i64::MAX, Pow2::<u64>::from_exponent(62).unwrap()),
            2
        );
        assert_eq!(
            div_round(i64::MAX, Pow2::<u64>::from_exponent(63).unwrap()),
            1
        );
        assert_eq!(
            div_round(u64::MAX, Pow2::<u64>::from_exponent(63).unwrap()),
            2
        );
    }

    #[test]
    fn unb_pow2_checked_div_round_boundary() {
        assert_eq!(
            checked_div_round(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_div_round(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div_round(0_i32, UnboundedPow2::from_exponent(32)),
            None
        );

        assert_eq!(
            checked_div_round(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_div_round(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_div_round_boundary() {
        assert_eq!(
            unbounded_div_round(0_i32, UnboundedPow2::from_exponent(30)),
            0
        );
        assert_eq!(
            unbounded_div_round(i32::MAX, UnboundedPow2::from_exponent(30)),
            2
        );
        assert_eq!(
            unbounded_div_round(0_i32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_div_round(i32::MIN, UnboundedPow2::from_exponent(31)),
            -1
        );
        assert_eq!(
            unbounded_div_round(i32::MIN, UnboundedPow2::from_exponent(32)),
            -1
        );
        assert_eq!(
            unbounded_div_round(-1_i32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_round(0_i32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_round(1_i32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_round(i32::MAX, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_round(-1_i32, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_round(i32::MIN, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_round(i32::MAX, UnboundedPow2::from_exponent(255)),
            0
        );

        assert_eq!(
            unbounded_div_round(0_u32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_div_round(0_u32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_div_round(0_u32, UnboundedPow2::from_exponent(255)),
            0
        );
        assert_eq!(
            unbounded_div_round(u32::MAX, UnboundedPow2::from_exponent(255)),
            0
        );
    }

    #[test]
    fn unb_pow2_floor_to_multiple_u64_already_aligned() {
        assert_eq!(
            floor_to_multiple(64u64, UnboundedPow2::from_exponent(6)),
            64
        );
    }

    #[test]
    fn pow2_floor_to_multiple_u64_already_aligned() {
        assert_eq!(
            floor_to_multiple(64u64, Pow2::<u8>::from_exponent(6).unwrap()),
            64
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_floor_to_multiple_align_out_of_range() {
        let _ = floor_to_multiple(64u64, UnboundedPow2::from_exponent(64));
    }

    #[test]
    fn unb_pow2_floor_to_multiple_u64_unaligned() {
        assert_eq!(
            floor_to_multiple(65u64, UnboundedPow2::from_exponent(6)),
            64
        );
        assert_eq!(
            floor_to_multiple(127u64, UnboundedPow2::from_exponent(6)),
            64
        );
    }

    #[test]
    fn pow2_floor_to_multiple_u64_unaligned() {
        assert_eq!(
            floor_to_multiple(65u64, Pow2::<u8>::from_exponent(6).unwrap()),
            64
        );
        assert_eq!(
            floor_to_multiple(127u64, Pow2::<u8>::from_exponent(6).unwrap()),
            64
        );
    }

    #[test]
    fn unb_pow2_floor_to_multiple_i64_positive() {
        assert_eq!(
            floor_to_multiple(37i64, UnboundedPow2::from_exponent(5)),
            32
        );
    }

    #[test]
    fn pow2_floor_to_multiple_i64_positive() {
        assert_eq!(
            floor_to_multiple(37i64, Pow2::<u8>::from_exponent(5).unwrap()),
            32
        );
    }

    #[test]
    fn unb_pow2_floor_to_multiple_i64_negative() {
        assert_eq!(
            floor_to_multiple(-37i64, UnboundedPow2::from_exponent(5)),
            -64
        );
        assert_eq!(
            floor_to_multiple(-32i64, UnboundedPow2::from_exponent(5)),
            -32
        );
    }

    #[test]
    fn pow2_floor_to_multiple_i64_negative() {
        assert_eq!(
            floor_to_multiple(-37i64, Pow2::<u8>::from_exponent(5).unwrap()),
            -64
        );
        assert_eq!(
            floor_to_multiple(-32i64, Pow2::<u8>::from_exponent(5).unwrap()),
            -32
        );
    }

    #[test]
    fn unb_pow2_floor_to_multiple_by_one_is_identity() {
        assert_eq!(
            floor_to_multiple(37i64, UnboundedPow2::from_exponent(0)),
            37
        );
        assert_eq!(
            floor_to_multiple(-37i64, UnboundedPow2::from_exponent(0)),
            -37
        );
    }

    #[test]
    fn pow2_floor_to_multiple_by_one_is_identity() {
        assert_eq!(
            floor_to_multiple(37i64, Pow2::<u8>::from_exponent(0).unwrap()),
            37
        );
        assert_eq!(
            floor_to_multiple(-37i64, Pow2::<u8>::from_exponent(0).unwrap()),
            -37
        );
    }

    #[test]
    fn unb_pow2_floor_to_multiple_min() {
        assert_eq!(
            floor_to_multiple(i32::MIN, UnboundedPow2::from_exponent(15)),
            i32::MIN
        );
        assert_eq!(
            floor_to_multiple(i32::MIN, UnboundedPow2::from_exponent(30)),
            i32::MIN
        );
        assert_eq!(
            floor_to_multiple(i32::MIN, UnboundedPow2::from_exponent(31)),
            i32::MIN
        );
    }

    #[test]
    fn pow2_floor_to_multiple_min() {
        assert_eq!(
            floor_to_multiple(i32::MIN, Pow2::<u32>::from_exponent(15).unwrap()),
            i32::MIN
        );
        assert_eq!(
            floor_to_multiple(i32::MIN, Pow2::<u32>::from_exponent(30).unwrap()),
            i32::MIN
        );
        assert_eq!(
            floor_to_multiple(i32::MIN, Pow2::<u32>::from_exponent(31).unwrap()),
            i32::MIN
        );
    }

    #[test]
    fn unb_pow2_floor_to_multiple_max() {
        assert_eq!(
            floor_to_multiple(i32::MAX, UnboundedPow2::from_exponent(30)),
            1 << 30
        );
        assert_eq!(
            floor_to_multiple(i32::MAX, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            floor_to_multiple(u32::MAX, UnboundedPow2::from_exponent(31)),
            1 << 31
        );
    }

    #[test]
    fn pow2_floor_to_multiple_max() {
        assert_eq!(
            floor_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(30).unwrap()),
            1 << 30
        );
        assert_eq!(
            floor_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            0
        );
        assert_eq!(
            floor_to_multiple(u32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            1 << 31
        );
    }

    #[test]
    fn unb_pow2_checked_floor_to_multiple_boundary() {
        assert_eq!(
            checked_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(32)),
            None
        );

        assert_eq!(
            checked_floor_to_multiple(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_floor_to_multiple(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_floor_to_multiple_boundary() {
        assert_eq!(
            unbounded_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            unbounded_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_floor_to_multiple(-1_i32, UnboundedPow2::from_exponent(31)),
            Some(i32::MIN)
        );
        assert_eq!(
            unbounded_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(32)),
            Some(0)
        );
        assert_eq!(
            unbounded_floor_to_multiple(-1_i32, UnboundedPow2::from_exponent(32)),
            None
        );
        assert_eq!(
            unbounded_floor_to_multiple(1_i32, UnboundedPow2::from_exponent(32)),
            Some(0)
        );
        assert_eq!(
            unbounded_floor_to_multiple(0_i32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );

        assert_eq!(
            unbounded_floor_to_multiple(0_u32, UnboundedPow2::from_exponent(31)),
            0
        );
        assert_eq!(
            unbounded_floor_to_multiple(0_u32, UnboundedPow2::from_exponent(32)),
            0
        );
        assert_eq!(
            unbounded_floor_to_multiple(0_u32, UnboundedPow2::from_exponent(255)),
            0
        );
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_u64_already_aligned() {
        assert_eq!(ceil_to_multiple(64u64, UnboundedPow2::from_exponent(6)), 64);
    }

    #[test]
    fn pow2_ceil_to_multiple_u64_already_aligned() {
        assert_eq!(
            ceil_to_multiple(64u64, Pow2::<u8>::from_exponent(6).unwrap()),
            64
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_ceil_to_multiple_align_out_of_range() {
        let _ = ceil_to_multiple(64u64, UnboundedPow2::from_exponent(64));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_ceil_to_multiple_overflow() {
        let _ = ceil_to_multiple(i32::MAX, UnboundedPow2::from_exponent(1));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn pow2_ceil_to_multiple_overflow() {
        let _ = ceil_to_multiple(i32::MAX, Pow2::<u8>::from_exponent(1).unwrap());
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_u64_unaligned() {
        assert_eq!(
            ceil_to_multiple(65u64, UnboundedPow2::from_exponent(6)),
            128
        );
        assert_eq!(ceil_to_multiple(1u64, UnboundedPow2::from_exponent(6)), 64);
    }

    #[test]
    fn pow2_ceil_to_multiple_u64_unaligned() {
        assert_eq!(
            ceil_to_multiple(65u64, Pow2::<u8>::from_exponent(6).unwrap()),
            128
        );
        assert_eq!(
            ceil_to_multiple(1u64, Pow2::<u8>::from_exponent(6).unwrap()),
            64
        );
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_u64_zero() {
        assert_eq!(ceil_to_multiple(0u64, UnboundedPow2::from_exponent(5)), 0);
    }

    #[test]
    fn pow2_ceil_to_multiple_u64_zero() {
        assert_eq!(
            ceil_to_multiple(0u64, Pow2::<u8>::from_exponent(5).unwrap()),
            0
        );
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_i64_positive() {
        assert_eq!(ceil_to_multiple(33i64, UnboundedPow2::from_exponent(5)), 64);
        assert_eq!(ceil_to_multiple(32i64, UnboundedPow2::from_exponent(5)), 32);
    }

    #[test]
    fn pow2_ceil_to_multiple_i64_positive() {
        assert_eq!(
            ceil_to_multiple(33i64, Pow2::<u8>::from_exponent(5).unwrap()),
            64
        );
        assert_eq!(
            ceil_to_multiple(32i64, Pow2::<u8>::from_exponent(5).unwrap()),
            32
        );
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_i64_negative() {
        assert_eq!(
            ceil_to_multiple(-33i64, UnboundedPow2::from_exponent(5)),
            -32
        );
        assert_eq!(
            ceil_to_multiple(-32i64, UnboundedPow2::from_exponent(5)),
            -32
        );
        assert_eq!(ceil_to_multiple(-16i64, UnboundedPow2::from_exponent(5)), 0);
    }

    #[test]
    fn pow2_ceil_to_multiple_i64_negative() {
        assert_eq!(
            ceil_to_multiple(-33i64, Pow2::<u8>::from_exponent(5).unwrap()),
            -32
        );
        assert_eq!(
            ceil_to_multiple(-32i64, Pow2::<u8>::from_exponent(5).unwrap()),
            -32
        );
        assert_eq!(
            ceil_to_multiple(-16i64, Pow2::<u8>::from_exponent(5).unwrap()),
            0
        );
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_min() {
        assert_eq!(
            ceil_to_multiple(i32::MIN, UnboundedPow2::from_exponent(2)),
            i32::MIN
        );
        assert_eq!(
            ceil_to_multiple(i32::MIN, UnboundedPow2::from_exponent(30)),
            i32::MIN
        );
    }

    #[test]
    fn pow2_ceil_to_multiple_min() {
        assert_eq!(
            ceil_to_multiple(i32::MIN, Pow2::<u32>::from_exponent(2).unwrap()),
            i32::MIN
        );
        assert_eq!(
            ceil_to_multiple(i32::MIN, Pow2::<u32>::from_exponent(30).unwrap()),
            i32::MIN
        );
    }

    #[test]
    fn unb_pow2_ceil_to_multiple_max() {
        assert_eq!(
            ceil_to_multiple(i32::MAX, UnboundedPow2::from_exponent(0)),
            i32::MAX
        );
        assert_eq!(
            ceil_to_multiple(i32::MAX - 1, UnboundedPow2::from_exponent(1)),
            i32::MAX - 1
        );

        assert_eq!(
            ceil_to_multiple(u32::MAX, UnboundedPow2::from_exponent(0)),
            u32::MAX
        );
        assert_eq!(
            ceil_to_multiple(u32::MAX - 1, UnboundedPow2::from_exponent(1)),
            u32::MAX - 1
        );
    }

    #[test]
    fn pow2_ceil_to_multiple_max() {
        assert_eq!(
            ceil_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(0).unwrap()),
            i32::MAX
        );
        assert_eq!(
            ceil_to_multiple(i32::MAX - 1, Pow2::<u32>::from_exponent(1).unwrap()),
            i32::MAX - 1
        );

        assert_eq!(
            ceil_to_multiple(u32::MAX, Pow2::<u32>::from_exponent(0).unwrap()),
            u32::MAX
        );
        assert_eq!(
            ceil_to_multiple(u32::MAX - 1, Pow2::<u32>::from_exponent(1).unwrap()),
            u32::MAX - 1
        );
    }

    #[test]
    fn unb_pow2_checked_ceil_to_multiple_boundary() {
        assert_eq!(
            checked_ceil_to_multiple(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_ceil_to_multiple(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_ceil_to_multiple(i32::MAX, UnboundedPow2::from_exponent(0)),
            Some(i32::MAX)
        );
        assert_eq!(
            checked_ceil_to_multiple(i32::MAX, UnboundedPow2::from_exponent(1)),
            None
        );

        assert_eq!(
            checked_ceil_to_multiple(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_ceil_to_multiple(i32::MAX as u32 + 2, UnboundedPow2::from_exponent(31)),
            None
        );
        assert_eq!(
            checked_ceil_to_multiple(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn pow2_checked_ceil_to_multiple_boundary() {
        assert_eq!(
            checked_ceil_to_multiple(0_i32, Pow2::<u32>::from_exponent(30).unwrap()),
            Some(0)
        );
        assert_eq!(
            checked_ceil_to_multiple(0_i32, Pow2::<u32>::from_exponent(31).unwrap()),
            Some(0)
        );
        assert_eq!(
            checked_ceil_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(0).unwrap()),
            Some(i32::MAX)
        );
        assert_eq!(
            checked_ceil_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(1).unwrap()),
            None
        );

        assert_eq!(
            checked_ceil_to_multiple(0_u32, Pow2::<u32>::from_exponent(31).unwrap()),
            Some(0)
        );
        assert_eq!(
            checked_ceil_to_multiple(i32::MAX as u32 + 2, Pow2::<u32>::from_exponent(31).unwrap()),
            None
        );
    }

    #[test]
    fn unb_pow2_unbounded_ceil_to_multiple_boundary() {
        assert_eq!(
            unbounded_ceil_to_multiple(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(-1_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(i32::MIN, UnboundedPow2::from_exponent(31)),
            Some(i32::MIN)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(1_i32, UnboundedPow2::from_exponent(31)),
            None
        );
        assert_eq!(
            unbounded_ceil_to_multiple(-1_i32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(0_i32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(1_i32, UnboundedPow2::from_exponent(255)),
            None
        );
        assert_eq!(
            unbounded_ceil_to_multiple(i32::MIN, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(i32::MAX, UnboundedPow2::from_exponent(0)),
            Some(i32::MAX)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(i32::MAX, UnboundedPow2::from_exponent(1)),
            None
        );

        assert_eq!(
            unbounded_ceil_to_multiple(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(1_u32, UnboundedPow2::from_exponent(31)),
            Some(1 << 31)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(0_u32, UnboundedPow2::from_exponent(32)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(1_u32, UnboundedPow2::from_exponent(32)),
            None
        );
        assert_eq!(
            unbounded_ceil_to_multiple(0_u32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_ceil_to_multiple(1_u32, UnboundedPow2::from_exponent(255)),
            None
        );
        assert_eq!(
            unbounded_ceil_to_multiple(i32::MAX as u32 + 2, UnboundedPow2::from_exponent(31)),
            None
        );
    }

    #[test]
    fn unb_pow2_checked_round_to_multiple_boundary() {
        assert_eq!(
            checked_round_to_multiple(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_round_to_multiple(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MAX, UnboundedPow2::from_exponent(0)),
            Some(i32::MAX)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MAX, UnboundedPow2::from_exponent(1)),
            None
        );
        assert_eq!(
            checked_round_to_multiple(i32::MIN, UnboundedPow2::from_exponent(1)),
            Some(i32::MIN)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MIN + 1, UnboundedPow2::from_exponent(1)),
            Some(i32::MIN)
        );

        assert_eq!(
            checked_round_to_multiple(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MAX as u32 + 2, UnboundedPow2::from_exponent(31)),
            Some(i32::MAX as u32 + 1)
        );
        assert_eq!(
            checked_round_to_multiple(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn pow2_checked_round_to_multiple_boundary() {
        assert_eq!(
            checked_round_to_multiple(0_i32, Pow2::<u32>::from_exponent(30).unwrap()),
            Some(0)
        );
        assert_eq!(
            checked_round_to_multiple(0_i32, Pow2::<u32>::from_exponent(31).unwrap()),
            Some(0)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(0).unwrap()),
            Some(i32::MAX)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MAX, Pow2::<u32>::from_exponent(1).unwrap()),
            None
        );
        assert_eq!(
            checked_round_to_multiple(i32::MIN, Pow2::<u32>::from_exponent(1).unwrap()),
            Some(i32::MIN)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MIN + 1, Pow2::<u32>::from_exponent(1).unwrap()),
            Some(i32::MIN)
        );

        assert_eq!(
            checked_round_to_multiple(0_u32, Pow2::<u32>::from_exponent(31).unwrap()),
            Some(0)
        );
        assert_eq!(
            checked_round_to_multiple(i32::MAX as u32 + 2, Pow2::<u32>::from_exponent(31).unwrap()),
            Some(i32::MAX as u32 + 1)
        );
    }

    #[test]
    fn unb_pow2_unbounded_round_to_multiple_boundary() {
        assert_eq!(
            unbounded_round_to_multiple(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(-1_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MIN, UnboundedPow2::from_exponent(31)),
            Some(i32::MIN)
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MIN, UnboundedPow2::from_exponent(32)),
            None
        );
        assert_eq!(
            unbounded_round_to_multiple(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(1_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(-1_i32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(0_i32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(1_i32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MIN, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MAX, UnboundedPow2::from_exponent(0)),
            Some(i32::MAX)
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MAX, UnboundedPow2::from_exponent(1)),
            None
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MAX as u32 + 1, UnboundedPow2::from_exponent(31)),
            Some(i32::MAX as u32 + 1)
        );
        assert_eq!(
            unbounded_round_to_multiple(i32::MAX as u32 + 1, UnboundedPow2::from_exponent(32)),
            None
        );

        assert_eq!(
            unbounded_round_to_multiple(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(0_u32, UnboundedPow2::from_exponent(32)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(1_u32, UnboundedPow2::from_exponent(32)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(0_u32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
        assert_eq!(
            unbounded_round_to_multiple(1_u32, UnboundedPow2::from_exponent(255)),
            Some(0)
        );
    }

    #[test]
    fn unb_pow2_rem_floor_u64_exact() {
        assert_eq!(rem_floor(32u64, UnboundedPow2::from_exponent(5)), 0);
    }

    #[test]
    fn pow2_rem_floor_u64_exact() {
        assert_eq!(rem_floor(32u64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic]
    fn unb_pow2_rem_floor_divisor_out_of_range() {
        let _ = rem_floor(32u64, UnboundedPow2::from_exponent(64));
    }

    #[test]
    fn unb_pow2_rem_floor_u64_remainder() {
        assert_eq!(rem_floor(37u64, UnboundedPow2::from_exponent(5)), 5);
        assert_eq!(rem_floor(63u64, UnboundedPow2::from_exponent(5)), 31);
    }

    #[test]
    fn pow2_rem_floor_u64_remainder() {
        assert_eq!(rem_floor(37u64, Pow2::<u8>::from_exponent(5).unwrap()), 5);
        assert_eq!(rem_floor(63u64, Pow2::<u8>::from_exponent(5).unwrap()), 31);
    }

    #[test]
    fn unb_pow2_rem_floor_i64_positive() {
        assert_eq!(rem_floor(37i64, UnboundedPow2::from_exponent(5)), 5);
    }

    #[test]
    fn pow2_rem_floor_i64_positive() {
        assert_eq!(rem_floor(37i64, Pow2::<u8>::from_exponent(5).unwrap()), 5);
    }

    #[test]
    fn unb_pow2_rem_floor_i64_negative() {
        assert_eq!(rem_floor(-1i64, UnboundedPow2::from_exponent(5)), 31);
    }

    #[test]
    fn pow2_rem_floor_i64_negative() {
        assert_eq!(rem_floor(-1i64, Pow2::<u8>::from_exponent(5).unwrap()), 31);
    }

    #[test]
    fn unb_pow2_rem_floor_min() {
        assert_eq!(rem_floor(i32::MIN, UnboundedPow2::from_exponent(2)), 0);
        assert_eq!(rem_floor(i32::MIN, UnboundedPow2::from_exponent(31)), 0);
    }

    #[test]
    fn pow2_rem_floor_min() {
        assert_eq!(
            rem_floor(i32::MIN, Pow2::<u8>::from_exponent(2).unwrap()),
            0
        );
        assert_eq!(
            rem_floor(i32::MIN, Pow2::<u32>::from_exponent(31).unwrap()),
            0
        );
    }

    #[test]
    fn unb_pow2_rem_floor_max() {
        assert_eq!(rem_floor(i32::MAX, UnboundedPow2::from_exponent(2)), 3);
        assert_eq!(
            rem_floor(i32::MAX, UnboundedPow2::from_exponent(31)),
            i32::MAX
        );
        assert_eq!(rem_floor(u32::MAX, UnboundedPow2::from_exponent(2)), 3);
        assert_eq!(
            rem_floor(u32::MAX, UnboundedPow2::from_exponent(31)),
            i32::MAX as u32
        );
    }

    #[test]
    fn pow2_rem_floor_max() {
        assert_eq!(
            rem_floor(i32::MAX, Pow2::<u8>::from_exponent(2).unwrap()),
            3
        );
        assert_eq!(
            rem_floor(i32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            i32::MAX
        );
        assert_eq!(
            rem_floor(u32::MAX, Pow2::<u8>::from_exponent(2).unwrap()),
            3
        );
        assert_eq!(
            rem_floor(u32::MAX, Pow2::<u32>::from_exponent(31).unwrap()),
            i32::MAX as u32
        );
    }

    #[test]
    fn unb_pow2_rem_floor_i64_negative_is_always_nonnegative() {
        // div_floor(-37, 32) = -64, so mod = -37 - (-64) = 27
        assert_eq!(rem_floor(-37i64, UnboundedPow2::from_exponent(5)), 27);
        // div_floor(-32, 32) = -32, so mod = 0
        assert_eq!(rem_floor(-32i64, UnboundedPow2::from_exponent(5)), 0);
        // div_floor(-1, 32) = -32, so mod = 31
        assert_eq!(rem_floor(-1i64, UnboundedPow2::from_exponent(5)), 31);
    }

    #[test]
    fn pow2_rem_floor_i64_negative_is_always_nonnegative() {
        // div_floor(-37, 32) = -64, so mod = -37 - (-64) = 27
        assert_eq!(rem_floor(-37i64, Pow2::<u8>::from_exponent(5).unwrap()), 27);
        // div_floor(-32, 32) = -32, so mod = 0
        assert_eq!(rem_floor(-32i64, Pow2::<u8>::from_exponent(5).unwrap()), 0);
        // div_floor(-1, 32) = -32, so mod = 31
        assert_eq!(rem_floor(-1i64, Pow2::<u8>::from_exponent(5).unwrap()), 31);
    }

    #[test]
    fn unb_pow2_checked_rem_floor_boundary() {
        assert_eq!(
            checked_rem_floor(0_i32, UnboundedPow2::from_exponent(30)),
            Some(0)
        );
        assert_eq!(
            checked_rem_floor(0_i32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_rem_floor(0_i32, UnboundedPow2::from_exponent(32)),
            None
        );

        assert_eq!(
            checked_rem_floor(0_u32, UnboundedPow2::from_exponent(31)),
            Some(0)
        );
        assert_eq!(
            checked_rem_floor(0_u32, UnboundedPow2::from_exponent(32)),
            None
        );
    }

    #[test]
    fn unb_pow2_is_multiple_of_u64_true() {
        assert!(is_multiple_of(0u64, UnboundedPow2::from_exponent(5)));
        assert!(is_multiple_of(32u64, UnboundedPow2::from_exponent(5)));
        assert!(is_multiple_of(64u64, UnboundedPow2::from_exponent(5)));
    }

    #[test]
    fn pow2_is_multiple_of_u64_true() {
        assert!(is_multiple_of(0u64, Pow2::<u64>::from_exponent(5).unwrap()));
        assert!(is_multiple_of(
            32u64,
            Pow2::<u64>::from_exponent(5).unwrap()
        ));
        assert!(is_multiple_of(
            64u64,
            Pow2::<u64>::from_exponent(5).unwrap()
        ));
    }

    #[test]
    fn unb_pow2_is_multiple_of_u64_false() {
        assert!(!is_multiple_of(31u64, UnboundedPow2::from_exponent(5)));
        assert!(!is_multiple_of(33u64, UnboundedPow2::from_exponent(5)));
    }

    #[test]
    fn pow2_is_multiple_of_u64_false() {
        assert!(!is_multiple_of(
            31u64,
            Pow2::<u64>::from_exponent(5).unwrap()
        ));
        assert!(!is_multiple_of(
            33u64,
            Pow2::<u64>::from_exponent(5).unwrap()
        ));
    }

    #[test]
    fn unb_pow2_is_multiple_of_i64_negative() {
        assert!(is_multiple_of(-32i64, UnboundedPow2::from_exponent(5)));
        assert!(is_multiple_of(i32::MIN, UnboundedPow2::from_exponent(31)));
        assert!(!is_multiple_of(-31i64, UnboundedPow2::from_exponent(5)));
    }

    #[test]
    fn pow2_is_multiple_of_i64_negative() {
        assert!(is_multiple_of(
            -32i64,
            Pow2::<u64>::from_exponent(5).unwrap()
        ));
        assert!(is_multiple_of(
            i32::MIN,
            Pow2::<u32>::from_exponent(31).unwrap()
        ));
        assert!(!is_multiple_of(
            -31i64,
            Pow2::<u64>::from_exponent(5).unwrap()
        ));
    }

    #[test]
    fn unb_pow2_unbounded_is_multiple_of_boundary() {
        assert!(unbounded_is_multiple_of(
            0_i32,
            UnboundedPow2::from_exponent(30)
        ));
        assert!(unbounded_is_multiple_of(
            i32::MIN,
            UnboundedPow2::from_exponent(31)
        ));
        assert!(unbounded_is_multiple_of(
            0_i32,
            UnboundedPow2::from_exponent(31)
        ));
        assert!(unbounded_is_multiple_of(
            0_i32,
            UnboundedPow2::from_exponent(32)
        ));
        assert!(unbounded_is_multiple_of(
            0_i32,
            UnboundedPow2::from_exponent(33)
        ));
        assert!(unbounded_is_multiple_of(
            0_i32,
            UnboundedPow2::from_exponent(255)
        ));
        assert!(!unbounded_is_multiple_of(
            i32::MIN,
            UnboundedPow2::from_exponent(255)
        ));
    }

    #[test]
    fn unb_pow2_div_floor_consistent_with_floor_to_multiple() {
        let b = UnboundedPow2::from_exponent(5); // 32
        for a in (-200i64..=200).step_by(7) {
            // floor_to_multiple gives the floored dividend times the divisor,
            // so recover the quotient via div_floor itself rather than `/`
            // (plain `/` truncates toward zero, not toward -∞).
            let q = div_floor(a, b);
            assert_eq!(floor_to_multiple(a, b), q * 32, "mismatch at a={a}");
        }
    }

    #[test]
    fn unb_pow2_floor_and_ceil_to_multiple_bracket_the_input() {
        let b = UnboundedPow2::from_exponent(4); // 16
        for a in 0u64..=200 {
            let lo = floor_to_multiple(a, b);
            let hi = ceil_to_multiple(a, b);
            assert!(lo <= a, "floor {lo} > {a}");
            assert!(hi >= a, "ceil {hi} < {a}");
            let gap = hi - lo;
            assert!(
                gap == 0 || gap == 16,
                "gap between floor and ceil should be 0 or 16, got {gap} for a={a}"
            );
        }
    }

    #[test]
    fn unb_pow2_rem_floor_plus_floor_to_multiple_equals_input() {
        let b = UnboundedPow2::from_exponent(5); // 32
        for a in (-300i64..=300).step_by(11) {
            let reconstructed = floor_to_multiple(a, b) + rem_floor(a, b);
            assert_eq!(reconstructed, a, "reconstruction failed at a={a}");
        }
    }

    #[test]
    fn unb_pow2_constants() {
        assert_eq!(UnboundedPow2::VAL_1, UnboundedPow2::from_exponent(0));
        assert_eq!(UnboundedPow2::VAL_2, UnboundedPow2::from_exponent(1));
        assert_eq!(UnboundedPow2::VAL_4, UnboundedPow2::from_exponent(2));
        assert_eq!(UnboundedPow2::VAL_8, UnboundedPow2::from_exponent(3));
        assert_eq!(UnboundedPow2::VAL_16, UnboundedPow2::from_exponent(4));
        assert_eq!(UnboundedPow2::VAL_32, UnboundedPow2::from_exponent(5));
        assert_eq!(UnboundedPow2::VAL_64, UnboundedPow2::from_exponent(6));
        assert_eq!(UnboundedPow2::VAL_128, UnboundedPow2::from_exponent(7));
        assert_eq!(UnboundedPow2::VAL_256, UnboundedPow2::from_exponent(8));
        assert_eq!(UnboundedPow2::VAL_512, UnboundedPow2::from_exponent(9));

        assert_eq!(UnboundedPow2::KIBI, UnboundedPow2::from_exponent(10));
        assert_eq!(UnboundedPow2::MEBI, UnboundedPow2::from_exponent(20));
        assert_eq!(UnboundedPow2::GIBI, UnboundedPow2::from_exponent(30));
        assert_eq!(UnboundedPow2::TEBI, UnboundedPow2::from_exponent(40));
        assert_eq!(UnboundedPow2::PEBI, UnboundedPow2::from_exponent(50));
        assert_eq!(UnboundedPow2::EXBI, UnboundedPow2::from_exponent(60));
        assert_eq!(UnboundedPow2::ZEBI, UnboundedPow2::from_exponent(70));
        assert_eq!(UnboundedPow2::YOBI, UnboundedPow2::from_exponent(80));
        assert_eq!(UnboundedPow2::ROBI, UnboundedPow2::from_exponent(90));
        assert_eq!(UnboundedPow2::QUEBI, UnboundedPow2::from_exponent(100));
    }

    #[test]
    fn unb_pow2_has_debug() {
        let v = UnboundedPow2::from_exponent(0);
        let mut s = String::new();
        write!(&mut s, "{:?}", v).unwrap();
        assert_eq!(s, "UnboundedPow2 { exponent: 0 }");
    }

    #[test]
    fn unb_pow2_has_hash() {
        let v = UnboundedPow2::from_exponent(123);
        let mut s0 = DefaultHasher::new();
        v.hash(&mut s0);
        let mut s1 = DefaultHasher::new();
        123u8.hash(&mut s1);
        assert_eq!(s0.finish(), s1.finish());
    }

    #[test]
    fn unb_pow2_has_eq() {
        assert_eq!(UnboundedPow2::VAL_1, UnboundedPow2::VAL_1);
        assert_eq!(UnboundedPow2::KIBI, UnboundedPow2::KIBI);
        assert_ne!(UnboundedPow2::VAL_1, UnboundedPow2::VAL_2);
    }

    #[test]
    fn unb_pow2_has_ord() {
        assert!(UnboundedPow2::VAL_1 < UnboundedPow2::VAL_2);
        assert!(UnboundedPow2::VAL_2 <= UnboundedPow2::VAL_2);
        assert!(UnboundedPow2::VAL_2 >= UnboundedPow2::VAL_2);
        assert!(UnboundedPow2::VAL_4 > UnboundedPow2::VAL_2);
    }
}
