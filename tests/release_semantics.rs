//! Tests for the crate's intentional debug-vs-release behavior split.
//!
//! Several normal (non-`checked`, non-`unbounded`) operations are documented to
//! panic in debug builds when a precondition is violated (overflow, or an
//! `UnboundedPow2` exponent too large for the left-hand-side width) and to
//! produce a "defined but incorrect" wrapped value in release builds.
//!
//! These tests exercise BOTH profiles: run `cargo test` for the debug-panic
//! contract and `cargo test --release` for the no-panic release paths. In every
//! case the corresponding `checked_*` API is asserted to report the failure as
//! `None`, which is the reliable, profile-independent contract.

use ipow2::{
    Pow2, UnboundedPow2, ceil_to_multiple, checked_ceil_to_multiple, checked_mul,
    checked_round_to_multiple, round_to_multiple,
};

#[cfg(debug_assertions)]
fn silence_panics() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| std::panic::set_hook(Box::new(|_| {})));
}

/// Assert that `$e` panics in debug builds and merely evaluates (no panic) in
/// release builds, exercising the code path in either profile.
macro_rules! debug_panics_release_ok {
    ($e:expr) => {{
        #[cfg(debug_assertions)]
        {
            silence_panics();
            assert!(
                std::panic::catch_unwind(|| $e).is_err(),
                "expected debug panic for: {}",
                stringify!($e)
            );
        }
        #[cfg(not(debug_assertions))]
        {
            let _ = $e;
        }
    }};
}

#[test]
fn ceil_to_multiple_overflow() {
    // ceil(200 / 128) = 2 -> 256, which does not fit a u8.
    let p = Pow2::<u8>::VAL_128;
    assert_eq!(checked_ceil_to_multiple(200u8, p), None);
    debug_panics_release_ok!(ceil_to_multiple(200u8, p));

    // Signed: ceil(100 / 128) = 1 -> 128 > i8::MAX.
    let q = Pow2::<u8>::VAL_128;
    assert_eq!(checked_ceil_to_multiple(100i8, q), None);
    debug_panics_release_ok!(ceil_to_multiple(100i8, q));
}

#[test]
fn round_to_multiple_overflow() {
    // round(200 / 128) = 2 -> 256, overflow.
    let p = Pow2::<u8>::VAL_128;
    assert_eq!(checked_round_to_multiple(200u8, p), None);
    debug_panics_release_ok!(round_to_multiple(200u8, p));
}

#[test]
fn mul_overflow() {
    let p2 = Pow2::<u8>::VAL_2;
    assert_eq!(checked_mul(200u8, p2), None);
    debug_panics_release_ok!(200u8 * p2);
    debug_panics_release_ok!({
        let mut a = 200u8;
        a *= p2;
        a
    });

    // Multiplying by an UnboundedPow2 whose exponent is unsafe for the width.
    let big = UnboundedPow2::from_exponent(9);
    assert_eq!(checked_mul(5u8, big), None);
    debug_panics_release_ok!(5u8 * big);
}

#[test]
fn unbounded_exponent_normal_ops_panic_in_debug() {
    // Exponent 9 is out of range for a u8 left-hand side; the normal division
    // operators debug-assert `is_safe` and panic, while `checked_*`/`unbounded_*`
    // stay well defined.
    let oob = UnboundedPow2::from_exponent(9);
    assert_eq!(ipow2::checked_div_floor(5u8, oob), None);
    assert_eq!(ipow2::unbounded_div_floor(5u8, oob), 0);
    debug_panics_release_ok!(ipow2::div_floor(5u8, oob));
    debug_panics_release_ok!(5u8 / oob);
    debug_panics_release_ok!(5u8 % oob);
}

#[test]
fn as_int_not_representable_panics_in_debug() {
    // 2^8 does not fit a u8, and 2^7 does not fit a (signed) i8.
    assert!(u8::try_from(UnboundedPow2::from_exponent(8)).is_err());
    assert!(i8::try_from(UnboundedPow2::from_exponent(7)).is_err());
    debug_panics_release_ok!(UnboundedPow2::from_exponent(8).as_u8());
    debug_panics_release_ok!(UnboundedPow2::from_exponent(7).as_i8());
}

/// A handful of exact wrapped values in release, documenting the "defined but
/// incorrect" behavior. Only compiled for optimized builds.
#[cfg(not(debug_assertions))]
#[test]
fn release_wrapped_values_are_defined() {
    // Multiplication wraps like `wrapping_shl` by the exponent.
    assert_eq!(200u8 * Pow2::<u8>::VAL_2, 200u8.wrapping_shl(1));
    assert_eq!(5u8 * UnboundedPow2::from_exponent(9), 5u8.wrapping_shl(9));
    // Out-of-range division shifts by the (bit-width-masked) exponent.
    assert_eq!(
        ipow2::div_floor(5u8, UnboundedPow2::from_exponent(9)),
        5u8.wrapping_shr(9)
    );
}
